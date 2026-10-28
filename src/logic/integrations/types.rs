pub enum MessageType {
    TEXT { text: String },
    PHOTO {},
    FILE {},
}
pub struct ReceivedMessage {
    pub id: i32,
    pub username: String,
    pub t: MessageType,
}
