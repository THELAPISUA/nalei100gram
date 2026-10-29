use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
