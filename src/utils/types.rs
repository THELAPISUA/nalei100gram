use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use teloxide::types::{ChatId, MessageId};

use crate::utils::types::MsgType::TEXT;

//
// Настройки юзера
//
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub user: User,
    pub theme: Option<Theme>,
    pub selected_theme: usize,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub token: String,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Theme {
    pub background: [u8; 4],
    pub surface: [u8; 4],
    pub accent: [u8; 4],
    pub text: [u8; 4],
    pub muted: [u8; 4],

    pub border: [u8; 4],
    pub hover: [u8; 4],
    pub input: [u8; 4],
    pub accent_text: [u8; 4],
}

pub struct PendingFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
}

#[derive(Clone)]
pub struct Chat {
    pub chat_id: ChatId,
    pub messages: Vec<MessagePacket>,
    pub unreaded_count: u32,
    pub name: String,
}
#[derive(Clone)]
pub enum MsgType {
    TEXT,
    IMAGE,
    STICKER,
    FILE,
    _AUDIO,
    _MUSIC,
    _VIDEO,
}

pub enum ActionPacket {
    Send(MessagePacket),
    Edit(Message),
    Delete(Message),

    GetChats(Option<Vec<Chat>>),
}
#[derive(Clone)]
pub struct Message {
    pub id: MessageId,
    pub chat_id: ChatId,
    pub new_text: Option<String>,
}

#[derive(Clone)]
pub struct MessagePacket {
    pub username: String,
    pub text: String,
    pub message_name: Option<String>,
    pub time: String,
    pub is_bot: bool,
    pub is_read: bool,
    pub msg_type: MsgType,
    pub chat_id: ChatId,
    pub message_id: Option<MessageId>,
    pub chat_name: Option<String>,
}
impl Default for MessagePacket {
    fn default() -> Self {
        Self {
            username: String::new(),
            text: String::new(),
            message_name: Some(String::new()),
            time: String::new(),
            is_bot: true,
            is_read: true,
            msg_type: TEXT,
            chat_id: ChatId(-1),
            message_id: None,
            chat_name: None,
        }
    }
}
