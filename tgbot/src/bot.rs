use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rand::{distributions::Alphanumeric, Rng};
use teloxide::{
    dptree::deps,
    payloads::SendMessage,
    prelude::*,
    RequestError,
    requests::JsonRequest,
    types::{KeyboardButton, KeyboardMarkup, ReplyMarkup},
};
use tokio::select;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument};

use crate::{
    config::{Action, CommandTrigger, ConfigType},
    server,
    server::ServerHandle,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Telegram request error: {0}")]
    RequestError(#[from] RequestError),

    #[error("Cancelled by another action")]
    Cancelled,

    #[error("Command not found")]
    CommandNotFound,

    #[error("Server error: {0}")]
    ServerError(#[from] server::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

struct Interpreter {
    cfg: ConfigType,
    bot: Bot,
    message: Message,
    delay: Delay,
    server: ServerHandle,
    tid: String,
    last_error: Option<Error>
}

impl Interpreter {
    fn new(
        cfg: ConfigType,
        bot: Bot,
        message: Message,
        delay: Delay,
        server: ServerHandle,
    ) -> Self {
        let tid = rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect();
        return Self { cfg, bot, message, delay, server, tid, last_error: None };
    }

    #[instrument(skip_all, fields(tid=self.tid, chat=%self.message.chat.id))]
    pub async fn execute(&mut self) {
        if let Err(error) = self.execute_fallible().await {
            if let Error::Cancelled = error {
                return;
            }

            if let Err(fatal) = self.handle_generic_error(error).await {
                self.handle_fatal_error(fatal);
            }
        }
    }

    async fn execute_fallible(&mut self) -> Result<()> {
        for (_, cmd) in &self.cfg.commands {
            if self.execute_trigger(&cmd.trigger)? {
                self.execute_actions(&cmd.actions).await?;
                return Ok(());
            }
        }

        Err(Error::CommandNotFound)
    }

    fn execute_trigger(&mut self, trigger: &'static CommandTrigger) -> Result<bool> {
        return match trigger {
            CommandTrigger::ExactMessage { text: expected } => match self.message.text() {
                None => Ok(false),

                Some(actual) => Ok(expected == actual),
            },
        };
    }

    async fn execute_actions(&mut self, actions: &'static Vec<Action>) -> Result<()> {
        for a in actions {
            self.execute_action(a).await?
        }
        Ok(())
    }

    async fn execute_action(&mut self, action: &'static Action) -> Result<()> {
        info!("execute_action {:?}", action);
        match action {
            Action::SendRawCommand { bytes } => {
                self.server.send_rc(bytes.as_slice()).await?;
                Ok(())
            }

            Action::Respond { text } => {
                self.action_respond(text.as_str()).await?;
                Ok(())
            }

            Action::Delay { key, seconds } => {
                self.delay.delay(*seconds, key.as_str()).await?;
                Ok(())
            }
        }
    }

    async fn handle_generic_error(&mut self, error: Error) -> Result<()> {
        error!("Error happened, falling back to generic error actions, error: {:?}", error);
        self.last_error = Some(error);
        self.execute_actions(&self.cfg.handlers.generic_error.actions).await?;
        Ok(())
    }

    fn handle_fatal_error(&mut self, error: Error) {
        error!("Fatal error happened: {:?}", error);
    }

    async fn action_respond(&mut self, template: &'static str) -> Result<()> {
        let text = self.render_template(template);
        let mut message = SendMessage::new(self.message.chat.id, text);
        let kb = &self.cfg.keyboard;

        message.reply_markup = Some(ReplyMarkup::Keyboard(KeyboardMarkup::new(
            kb.iter().map(|row| row.iter().map(|text| KeyboardButton::new(text))),
        )));

        JsonRequest::new(self.bot.clone(), message).await?;
        Ok(())
    }

    fn render_template(&mut self, template: &'static str) -> String {
        let mut result = String::from(template);

        if result.contains("{error}") && self.last_error.is_some() {
            let err = self.last_error.as_ref().unwrap();
            result = result.replace("{error}", &self.format_error(err))
        }

        if result.contains("{statusMessage}") {
            let message = if self.server.is_online() {
                self.cfg.server.online_message.as_str()
            } else {
                self.cfg.server.offline_message.as_str()
            };

            result = result.replace("{statusMessage}", message)
        }

        return result;
    }

    fn format_error(&self, error: &Error) -> String {
        let m = &self.cfg.messages;

        match error {
            Error::RequestError(_) => { m.telegram_error.clone() }
            Error::Cancelled => {m.command_canceled.clone() }
            Error::CommandNotFound => { m.command_not_found.clone() }
            Error::ServerError(server) => {
                match server {
                    server::Error::NotImplemented => { m.not_implemented.clone() }
                    server::Error::Io(io) => { format!("{}: {:?}", m.internal_error, io) }
                    server::Error::MpcResponseTimeout => { format!("{}: {:?}", m.internal_error, "MpcResponseTimeout") }
                    server::Error::IRAckTimeout => { m.no_response_from_rc.clone() }
                    server::Error::MpcGenericError(msg) => { format!("{}: {:?}", m.internal_error, msg) }
                    server::Error::CanNotSendWhenOffline => { m.can_not_send_when_offline.clone() }
                    server::Error::JsonError(er) => { format!("{}: {:?}", m.internal_error, er) }
                }
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct Delay {
    cts: Arc<Mutex<HashMap<&'static str, CancellationToken>>>,
}

impl Delay {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn delay(&self, seconds: u64, key: &'static str) -> Result<()> {
        let ct = CancellationToken::new();
        let option = self.cts.lock().unwrap().insert(key, ct.clone());

        if let Some(ct) = option {
            info!(
                "Cancelling other delays for key={} and scheduling a new one with delay {} seconds",
                key, seconds
            );
            ct.cancel();
        }

        select! {
            _ = ct.cancelled() => {
                info!("Delay cancelled by other action");
                Err(Error::Cancelled)
            }

            _ = sleep(std::time::Duration::from_secs(seconds)) => {
                info!("Delay successfully finished");
                self.cts.lock().unwrap().remove(key);
                Ok(())
            }
        }
    }
}

pub async fn run(cfg: ConfigType, server: ServerHandle) {
    let bot = Bot::from_env();
    let delay = Delay::new();

    Dispatcher::builder(
        bot,
        Update::filter_message().endpoint(
            move |bot: Bot, message: Message, delay: Delay, server: ServerHandle| async move {
                let mut interpreter = Interpreter::new(cfg, bot, message, delay, server);

                tokio::spawn(async move {
                    interpreter.execute().await;
                });

                Result::Ok(())
            },
        ),
    )
        .dependencies(deps![delay, server])
        .build()
        .dispatch()
        .await;
}
