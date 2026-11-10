pub enum VoiceAction {
    None,
    PlayPause,
    Seek(f64),
}
pub enum MessageAction {
    Edit,
    Copy,
    Delete,
    Open,
    Clicked,
}
