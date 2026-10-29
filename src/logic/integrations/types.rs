#[derive(Clone)]
pub enum MessageType {
    TEXT { text: String },
    PHOTO { path: String },
    FILE { path: String, name: String },
    STICKER { path: String },
}
#[derive(Clone)]
pub struct ReceivedMessage {
    pub id: i32,
    pub username: String,
    pub t: MessageType,
    pub date: String,
    pub chat_id: i64,
    pub readed: bool,
    pub sender_is_you: bool,
    pub account_id: i64,
}

#[derive(Clone)]
pub struct Chat {
    pub chat_id: i64,
    pub messages: Vec<ReceivedMessage>,
    pub unreaded_count: u32,
    pub name: String,
}

pub enum ActionPacket {
    Send(ReceivedMessage),
    Edit {
        accaunt_id: i64,
        chat_id: i64,
        id: i32,
        new_text: String,
    },
    Delete {
        id: i32,
        account_id: i64,
        chat_id: i64,
    },

    AccountInfo(String, i64),
}
