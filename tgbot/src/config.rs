use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const CONFIG: &[u8] = include_bytes!("../config.yaml");

pub type ConfigType = &'static Config;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub version: u32,
    pub keyboard: Vec<Vec<String>>,
    pub commands: HashMap<String, Command>,
    pub handlers: Handlers,
    pub server: ServerConfig,
    pub messages: Messages,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Messages {
    pub not_implemented: String,
    pub command_not_found: String,
    pub telegram_error: String,
    pub command_canceled: String,
    pub internal_error: String,
    pub no_response_from_rc: String,
    pub can_not_send_when_offline: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub online_message: String,
    pub offline_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub trigger: CommandTrigger,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CommandTrigger {
    ExactMessage { text: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    SendRawCommand { bytes: Vec<u32> },
    Respond { text: String },
    Delay { seconds: u64, key: String },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Handlers {
    pub generic_error: ActionsOnly,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionsOnly {
    pub actions: Vec<Action>,
}

lazy_static! {
    static ref CONFIG_PARSED: Config = {
        let s = std::str::from_utf8(CONFIG).unwrap();
        serde_yaml::from_str(s).unwrap()
    };
}

pub fn get_config() -> ConfigType {
    &CONFIG_PARSED
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u64,
    pub name: String,
}

pub type UsersType = &'static Vec<User>;

lazy_static! {
    static ref USERS_PARSED: Vec<User> = {
        let allow_access = std::env::var("BOT_AUTHORIZED_USERS")
            .expect("env variable BOT_AUTHORIZED_USERS should be defined");
        serde_yaml::from_str(&allow_access).unwrap()
    };
}

pub fn get_users() -> UsersType {
    &USERS_PARSED
}
