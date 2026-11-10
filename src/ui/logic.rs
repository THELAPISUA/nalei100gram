use crate::{
    logic::integrations::types::{
        ActionPacket::{self, *},
        Chat,
        MessageType::{self, TEXT},
        ReceivedMessage,
    },
    ui::app::GramApp,
    utils::types::PendingFile,
};
use arboard::{Clipboard, ImageData};
use chrono::Local;
use eframe::egui;

use log::info;
use std::{borrow::Cow, ffi::OsStr, path::PathBuf};
use tokio::sync::mpsc::Sender;

impl GramApp {
    pub(crate) fn get_or_create_chat(&mut self, chat_id: i64, name: String) -> &mut Chat {
        if let Some(index) = self.chats.iter().position(|chat| chat.chat_id == chat_id) {
            return &mut self.chats[index];
        }

        self.chats.push(Chat {
            chat_id,
            messages: Vec::new(),
            unreaded_count: 0,
            name: name,
        });

        self.chats.last_mut().unwrap()
    }
    pub(crate) fn recv_messages(&mut self) {
        while let Ok(packet) = self.rx.try_recv() {
            match packet {
                Send(m) => {
                    let chat_id = m.chat_id;
                    let chat = self.get_or_create_chat(chat_id, m.username.clone());

                    chat.unreaded_count += 1;
                    chat.messages.push(m.clone());
                }
                AccountInfo(name, id) => {
                    if let Some(account) = self.accounts.iter_mut().find(|i| i.0 == id) {
                        account.1 = name;
                    }
                }
                _ => {}
            }
        }
    }
    pub(crate) fn get_last_id(&self, curr_chat: i64) -> i32 {
        self.chats
            .iter()
            .find(|c| c.chat_id == curr_chat)
            .and_then(|chat| chat.messages.last())
            .map(|message| message.id)
            .unwrap_or(0)
    }
    pub(crate) fn sender_docs_messages(&mut self) {
        while let Some(file) = self.pending_files.pop() {
            self.process_file(file);
        }
    }
    pub(crate) fn cancel_close(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|i| {
            i.viewport()
                .events
                .iter()
                .any(|event| matches!(event, egui::ViewportEvent::Close))
        });

        if close_requested {
            #[cfg(target_os = "windows")]
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }
        }
    }
    pub(crate) fn collect_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());

        for file in dropped {
            let path = file.path().to_path_buf();

            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

            let extension = path
                .extension()
                .unwrap_or(OsStr::new("dat"))
                .to_str()
                .unwrap()
                .to_string();

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            info!("collected file: {}", name);

            self.pending_files.push(PendingFile {
                name,
                path,
                size,
                extension,
            });
        }
    }
    pub(crate) fn last_message(&mut self, chad_id: i64, current_account_id: i64) -> String {
        let curr_chat = self
            .chats
            .iter()
            .find(|c| c.chat_id == chad_id)
            .map(|c| &c.messages);

        let last_msg_p: Option<&ReceivedMessage> = curr_chat.as_ref().and_then(|messages| {
            messages
                .iter()
                .rfind(|a| a.account_id == current_account_id || a.chat_id == 0)
        });

        match last_msg_p {
            Some(m) => match m.t {
                MessageType::TEXT { ref text } => text.clone(),
                MessageType::PHOTO {
                    path: _,
                    caption: _,
                } => String::from("Фото"),
                MessageType::STICKER { path: _ } => String::from("Стикер"),
                MessageType::FILE {
                    path: _,
                    name: _,
                    caption: _,
                } => String::from("Файл"),
                MessageType::AUDIO {
                    path: _,
                    name: _,
                    caption: _,
                } => String::from("Аудіо"),
                MessageType::VOICE { path: _ } => String::from("Голосове"),
                // _ => String::from("Невідомий формат"),
            },
            None => String::from("Поки нема повідомлень!"),
        }
    }
    pub(crate) fn send_message_packet(&mut self, msg: String) {
        let curr_id = if self.current_chat == 0 {
            -1
        } else {
            self.current_account_id
        };
        let packet = ReceivedMessage {
            date: Local::now().format("%H:%M").to_string(),
            username: String::from("You"),
            t: MessageType::TEXT { text: msg },
            chat_id: self.current_chat.clone(),
            sender_is_you: true,
            readed: true,
            id: self.get_last_id(self.current_chat.clone()) + 1,
            account_id: curr_id,
        };
        if self.tx.try_send(Send(packet.clone())).is_ok() {
            let current_chat = self.current_chat.clone();
            let chat = self.get_or_create_chat(current_chat.clone(), String::new());

            chat.messages.push(packet);

            self.input_text.clear();
        }
    }
    pub fn process_file(&mut self, file: PendingFile) {
        //40 мб це ліміт на бота в тг
        if file.size <= 41943040 {
            let message_type = match file.extension.as_str() {
                "mp3" | "wav" | "ogg" => MessageType::AUDIO {
                    path: file.path.to_string_lossy().to_string(),
                    name: file.name,
                    caption: None,
                },
                "png" | "jpg" | "webp" => MessageType::PHOTO {
                    path: file.path.to_string_lossy().to_string(),
                    caption: None,
                },
                _ => MessageType::FILE {
                    path: file.path.to_string_lossy().to_string(),
                    name: file.name,
                    caption: None,
                },
            };
            let curr_id = if self.current_chat == 0 {
                -1
            } else {
                self.current_account_id
            };
            let msg = ReceivedMessage {
                date: Local::now().format("%H:%M").to_string(),
                username: String::from("You"),
                t: message_type,
                chat_id: self.current_chat.clone(),
                sender_is_you: true,
                readed: true,
                id: self.get_last_id(self.current_chat.clone()) + 1,
                account_id: curr_id,
            };
            let is_ok = self.tx.try_send(Send(msg.clone())).is_ok();
            if is_ok {
                let current_chat = self.current_chat.clone();
                let chat = self.get_or_create_chat(current_chat.clone(), String::new());
                chat.messages.push(msg);
            }
        }
    }

    pub fn select_and_send_file(&mut self) {
        if let Some(mut path) = rfd::FileDialog::new().pick_files() {
            while let Some(file) = path.pop() {
                let file = pathbuf_to_pendingfile(file);
                self.process_file(file);
            }
        }
    }
}
pub(crate) fn edit_message(msg_id: i32, account_id: i64, chat: &mut Chat, new_text: String) {
    let msg = get_mut_message_by_id(msg_id, account_id, chat).unwrap();
    match &mut msg.t {
        TEXT { text } => *text = new_text,
        _ => {}
    }
}

pub(crate) fn delete_msg(msg_id: i32, account_id: i64, chat: &mut Chat, tx: &Sender<ActionPacket>) {
    let mut account_id = account_id;
    if chat.chat_id == 0 {
        account_id = -1;
    }

    chat.messages
        .retain(|m| !(m.id == msg_id && m.account_id == account_id));
    let _ = tx.try_send(Delete {
        id: msg_id,
        account_id,
        chat_id: chat.chat_id,
    });
}
pub(crate) fn copy_text_to_clipboard(text: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(text).unwrap();
}
pub fn line_break(s: &str) -> String {
    let mut ans = String::new();

    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % 50 == 0 {
            ans.push('\n');
        }

        ans.push(ch);
    }

    ans
}
pub(crate) fn copy_photo_to_clipboard(path: &str) {
    let img = image::open(path).unwrap().to_rgba8();
    let (width, height) = img.dimensions();

    let image_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: Cow::Owned(img.into_raw()),
    };

    let mut clipboard = Clipboard::new().unwrap();
    let _ = clipboard.set_image(image_data);
}
pub fn pathbuf_to_pendingfile(f: PathBuf) -> PendingFile {
    PendingFile {
        name: f
            .file_name()
            .unwrap_or(OsStr::new("File"))
            .to_string_lossy()
            .to_string(),
        path: f.clone(),
        size: f.metadata().unwrap().len(),
        extension: f
            .extension()
            .unwrap_or(OsStr::new("dat"))
            .to_string_lossy()
            .to_string(),
    }
}
pub fn get_message_by_id(id: i32, account_id: i64, chat: &Chat) -> Option<&ReceivedMessage> {
    chat.messages
        .iter()
        .find(|m| m.id == id && (m.account_id == account_id || m.account_id == -1))
}
pub fn get_mut_message_by_id(
    id: i32,
    account_id: i64,
    chat: &mut Chat,
) -> Option<&mut ReceivedMessage> {
    chat.messages
        .iter_mut()
        .find(|m| m.id == id && (m.account_id == account_id || m.account_id == -1))
}
