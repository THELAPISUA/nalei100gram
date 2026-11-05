pub enum VoiceAction {
    None,
    PlayPause,
    Seek(f64),
}
pub enum MessageAction {
    _Edit,
    Copy,
    Delete,
    Open,
    Clicked,
}
