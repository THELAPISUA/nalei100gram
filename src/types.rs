use teloxide::types::ChatId;

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
    pub msgtype: MsgType,
    pub chat_id: ChatId,
}
