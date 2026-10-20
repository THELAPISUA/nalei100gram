use std::path::PathBuf;
use teloxide::types::ChatId;

use crate::utils::types::MsgType::TEXT;
pub struct PendingFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
}
pub struct Chat {
    pub chat_id: ChatId,
    pub messages: Vec<MessagePacket>,
    pub unreaded_count: u32,
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

#[derive(Clone)]
pub struct MessagePacket {
    pub username: Option<String>,
    pub text: String,
    pub msg_name: Option<String>,
    pub time: String,
    pub is_bot: bool,
    pub is_read: bool,
    pub msg_type: MsgType,
    pub chat_id: ChatId,
}
impl Default for MessagePacket {
    fn default() -> Self {
        Self {
            username: Some(String::new()),
            text: String::new(),
            msg_name: Some(String::new()),
            time: String::new(),
            is_bot: true,
            is_read: true,
            msg_type: TEXT,
            chat_id: ChatId(-1),
        }
    }
}
