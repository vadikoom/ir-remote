use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rand::{distr::Alphanumeric, RngExt};
use teloxide::{
    dptree::deps,
    payloads::SendMessage,
    prelude::*,
    requests::JsonRequest,
    types::{KeyboardButton, KeyboardMarkup, ReplyMarkup},
    RequestError,
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
    Server(#[from] server::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

struct Interpreter {
    cfg: ConfigType,
    bot: Bot,
    message: Message,
    delay: Delay,
    server: ServerHandle,
    tid: String,
    last_error: Option<Error>,
}

impl Interpreter {
    fn new(
        cfg: ConfigType,
        bot: Bot,
        message: Message,
        delay: Delay,
        server: ServerHandle,
    ) -> Self {
        let tid = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        Self {
            cfg,
            bot,
            message,
            delay,
            server,
            tid,
            last_error: None,
        }
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
        for cmd in self.cfg.commands.values() {
            if self.execute_trigger(&cmd.trigger)? {
                self.execute_actions(&cmd.actions).await?;
                return Ok(());
            }
        }

        Err(Error::CommandNotFound)
    }

    fn execute_trigger(&mut self, trigger: &'static CommandTrigger) -> Result<bool> {
        match trigger {
            CommandTrigger::ExactMessage { text: expected } => match self.message.text() {
                None => Ok(false),

                Some(actual) => Ok(expected == actual),
            },
        }
    }

    async fn execute_actions(&mut self, actions: &'static [Action]) -> Result<()> {
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
        error!(
            "Error happened, falling back to generic error actions, error: {:?}",
            error
        );
        self.last_error = Some(error);
        self.execute_actions(&self.cfg.handlers.generic_error.actions)
            .await?;
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
            kb.iter().map(|row| row.iter().map(KeyboardButton::new)),
        )));

        JsonRequest::new(self.bot.clone(), message).await?;
        Ok(())
    }

    fn render_template(&mut self, template: &'static str) -> String {
        let mut result = String::from(template);

        if let Some(err) = &self.last_error {
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

        result
    }

    fn format_error(&self, error: &Error) -> String {
        let m = &self.cfg.messages;

        match error {
            Error::RequestError(_) => m.telegram_error.clone(),
            Error::Cancelled => m.command_canceled.clone(),
            Error::CommandNotFound => m.command_not_found.clone(),
            Error::Server(server) => match server {
                server::Error::Security(_) => m.internal_error.clone(),
                server::Error::Io(io) => {
                    format!("{}: {:?}", m.internal_error, io)
                }
                server::Error::MpcResponseTimeout => {
                    format!("{}: {:?}", m.internal_error, "MpcResponseTimeout")
                }
                server::Error::CommandSuperseded => m.command_canceled.clone(),
                server::Error::IRAckTimeout => m.no_response_from_rc.clone(),
                server::Error::MpcChannel(msg) => {
                    format!("{}: {:?}", m.internal_error, msg)
                }
                server::Error::CanNotSendWhenOffline => m.can_not_send_when_offline.clone(),
                server::Error::Json(er) => {
                    format!("{}: {:?}", m.internal_error, er)
                }
            },
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
        {
            let mut timers = self.cts.lock().unwrap();
            if let Some(ct) = timers.insert(key, ct.clone()) {
                info!(
                    key,
                    seconds, "Cancelling previous delay and scheduling its replacement"
                );
                ct.cancel();
            }
        }

        select! {
            _ = ct.cancelled() => {
                info!("Delay cancelled by other action");
                Err(Error::Cancelled)
            }

            _ = sleep(std::time::Duration::from_secs(seconds)) => {
                let mut timers = self.cts.lock().unwrap();
                if ct.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                timers.remove(key);
                info!("Delay successfully finished");
                Ok(())
            }
        }
    }
}

fn is_authorized(sender: Option<u64>, users: &[crate::config::User]) -> bool {
    sender.is_some_and(|id| users.iter().any(|user| user.id == id))
}

pub async fn run(cfg: ConfigType, server: ServerHandle) {
    let bot = Bot::from_env();
    let delay = Delay::new();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter(|message: Message| {
                is_authorized(
                    message.from.as_ref().map(|user| user.id.0),
                    crate::config::get_users(),
                )
            })
            .endpoint(
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::poll;
    use std::time::Duration;

    #[test]
    fn authorization_requires_a_known_sender() {
        let users = vec![
            crate::config::User {
                id: 10,
                name: "First".into(),
            },
            crate::config::User {
                id: 20,
                name: "Second".into(),
            },
        ];
        assert!(is_authorized(Some(10), &users));
        assert!(is_authorized(Some(20), &users));
        assert!(!is_authorized(Some(30), &users));
        assert!(!is_authorized(None, &users));
        assert!(!is_authorized(Some(10), &[]));
    }

    #[tokio::test]
    async fn status_command_sends_expected_text_and_keyboard_to_telegram() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let cfg = crate::config::get_config();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let api_url = format!("http://{}/", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let message_json = serde_json::json!({
            "message_id": 1, "date": 0,
            "chat": {"id": 10, "type": "private", "first_name": "Test"},
            "from": {"id": 10, "is_bot": false, "first_name": "Test"},
            "text": "статус"
        });
        let message: Message = serde_json::from_value(message_json.clone()).unwrap();
        let mock = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 4096];
            let (headers, body_start, content_length) = loop {
                let count = socket.read(&mut buffer).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&buffer[..count]);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let headers = String::from_utf8(request[..end].to_vec()).unwrap();
                    let content_length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    break (headers, end + 4, content_length);
                }
            };
            while request.len() < body_start + content_length {
                let count = socket.read(&mut buffer).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&buffer[..count]);
            }
            assert!(headers
                .to_ascii_lowercase()
                .starts_with("post /bot12345:test/sendmessage "));
            let body: serde_json::Value =
                serde_json::from_slice(&request[body_start..body_start + content_length]).unwrap();
            let response = serde_json::json!({"ok": true, "result": message_json}).to_string();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response);
            socket.write_all(response.as_bytes()).await.unwrap();
            body
        });
        let (_runner, handle) = server::new(
            std::net::Ipv4Addr::LOCALHOST,
            0,
            server::crypto::Crypto::from_hex(server::crypto::TEST_SECRET).unwrap(),
        )
        .await
        .unwrap();
        let bot = Bot::new("12345:TEST").set_api_url(api_url);
        let mut interpreter = Interpreter::new(cfg, bot, message, Delay::new(), handle);
        tokio::time::timeout(Duration::from_secs(5), interpreter.execute_fallible())
            .await
            .unwrap()
            .unwrap();
        let body = mock.await.unwrap();
        assert_eq!(body["chat_id"], 10);
        assert_eq!(body["text"], cfg.server.offline_message);
        let keyboard = body["reply_markup"]["keyboard"].as_array().unwrap();
        assert_eq!(keyboard.len(), cfg.keyboard.len());
        for (actual_row, expected_row) in keyboard.iter().zip(&cfg.keyboard) {
            let actual: Vec<_> = actual_row
                .as_array()
                .unwrap()
                .iter()
                .map(|button| button["text"].as_str().unwrap())
                .collect();
            assert_eq!(
                actual,
                expected_row.iter().map(String::as_str).collect::<Vec<_>>()
            );
        }
    }

    #[tokio::test(start_paused = true)]
    async fn expired_timer_cannot_complete_after_replacement() {
        let delay = Delay::new();
        let old = delay.delay(1, "off");
        tokio::pin!(old);
        assert!(poll!(old.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(1)).await;

        let replacement = delay.delay(3600, "off");
        tokio::pin!(replacement);
        assert!(poll!(replacement.as_mut()).is_pending());
        assert!(matches!(old.await, Err(Error::Cancelled)));
        assert!(delay.cts.lock().unwrap().contains_key("off"));

        delay.delay(0, "off").await.unwrap();
        assert!(matches!(replacement.await, Err(Error::Cancelled)));
        assert!(delay.cts.lock().unwrap().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn different_timer_keys_do_not_cancel_each_other() {
        let delay = Delay::new();
        let first = delay.delay(1, "first");
        let second = delay.delay(2, "second");
        let (first, second) = tokio::join!(first, second);
        first.unwrap();
        second.unwrap();
        assert!(delay.cts.lock().unwrap().is_empty());
    }
}
