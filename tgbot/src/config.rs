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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_signals_encode_coolix_commands_in_both_frames() {
        let cfg = get_config();
        let command = &cfg.commands["Heat28"];
        let CommandTrigger::ExactMessage { text } = &command.trigger;
        assert_eq!(text, "☀️+28");
        assert!(cfg.keyboard.iter().flatten().any(|label| label == text));

        for (name, expected) in [
            ("Heat28", [0xB2, 0x4D, 0x9F, 0x60, 0x8C, 0x73]),
            ("Deh20", [0xB2, 0x4D, 0x1F, 0xE0, 0x24, 0xDB]),
            ("Cool20", [0xB2, 0x4D, 0x9F, 0x60, 0x20, 0xDF]),
            ("Cool24", [0xB2, 0x4D, 0x9F, 0x60, 0x40, 0xBF]),
        ] {
            assert_signal(&cfg.commands[name], expected);
        }
    }

    fn assert_signal(command: &Command, expected: [u8; 6]) {
        let bytes = match &command.actions[0] {
            Action::SendRawCommand { bytes } => bytes,
            _ => panic!("Mode commands must send an IR signal"),
        };
        assert_eq!(bytes.len(), 199);

        // COOLIX sends each byte MSB first, followed by its inverse.
        // https://github.com/crankyoldgit/IRremoteESP8266/blob/master/src/ir_Coolix.h
        for frame_start in [0, 100] {
            let mut decoded = Vec::new();
            for byte_index in 0..6 {
                let mut byte = 0u8;
                for bit_index in 0..8 {
                    let offset = frame_start + 2 + (byte_index * 8 + bit_index) * 2;
                    assert!((300..=700).contains(&bytes[offset]));
                    byte = (byte << 1) | u8::from(bytes[offset + 1] > 1000);
                }
                decoded.push(byte);
            }
            assert_eq!(decoded, expected);
        }
    }

    #[test]
    fn removed_controls_have_no_keyboard_buttons_or_triggers() {
        let cfg = get_config();
        for removed in ["🔴⏳15м", "💧+24"] {
            assert!(!cfg.keyboard.iter().flatten().any(|label| label == removed));
            assert!(!cfg.commands.values().any(|command| {
                let CommandTrigger::ExactMessage { text } = &command.trigger;
                text == removed
            }));
        }
    }
}
