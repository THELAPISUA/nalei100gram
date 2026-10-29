use crate::{
    logic::integrations::types::{ActionPacket::*, Chat, MessageType, ReceivedMessage},
    ui::app::GramApp,
    utils::types::PendingFile,
};
use arboard::Clipboard;
use chrono::Local;
use eframe::egui;

use log::info;
use std::ffi::OsStr;

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
                    self.name = name;
                }
                _ => {}
            }
        }
    }
    pub(crate) fn sender_docs_messages(&mut self) {
        while let Some(file) = self.pending_files.pop() {
            //40 мб це ліміт на бота в тг
            if file.size <= 41943040 {
                let message_type = match file.extension.as_str() {
                    "png" | "jpg" | "webp" => MessageType::PHOTO {
                        path: file.path.to_string_lossy().to_string(),
                    },
                    _ => MessageType::FILE {
                        path: file.path.to_string_lossy().to_string(),
                        name: String::new(),
                    },
                };
                let msg = ReceivedMessage {
                    date: Local::now().format("%H:%M").to_string(),
                    username: String::from("You"),
                    t: message_type,
                    chat_id: self.current_chat.clone(),
                    sender_is_you: true,
                    readed: true,
                    id: 1,
                    account_id: self.current_account_id,
                };
                let is_ok = self.tx.try_send(Send(msg.clone())).is_ok();
                if is_ok {
                    let current_chat = self.current_chat.clone();
                    let chat = self.get_or_create_chat(current_chat.clone(), String::new());
                    chat.messages.push(msg);
                }
            }
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
    pub(crate) fn last_message(&mut self, chad_id: i64) -> String {
        let curr_chat = self
            .chats
            .iter()
            .find(|c| c.chat_id == chad_id)
            .map(|c| &c.messages);

        let last_msg_p = curr_chat.and_then(|messages| messages.last());

        match last_msg_p {
            Some(m) => match m.t {
                MessageType::TEXT { ref text } => text.clone(),
                MessageType::PHOTO { path: _ } => String::from("Фото"),
                MessageType::STICKER { path: _ } => String::from("Стикер"),
                MessageType::FILE { path: _, name: _ } => String::from("ФАЙЛ"),
                _ => String::from("Невідомий формат"),
            },
            None => String::from("Поки нема повідомлень!"),
        }
    }
    pub(crate) fn send_message_packet(&mut self, msg: String) {
        let packet = ReceivedMessage {
            date: Local::now().format("%H:%M").to_string(),
            username: String::from("You"),
            t: MessageType::TEXT { text: msg },
            chat_id: self.current_chat.clone(),
            sender_is_you: true,
            readed: true,
            id: 0,
            account_id: self.current_account_id,
        };
        if self.tx.try_send(Send(packet.clone())).is_ok() {
            let current_chat = self.current_chat.clone();
            let chat = self.get_or_create_chat(current_chat.clone(), String::new());

            chat.messages.push(packet);

            self.input_text.clear();
        }
    }
}

pub(crate) fn delete_msg(msg_id: i32, account_id: i64, chat: &mut Chat) {
    chat.messages
        .retain(|m| m.id != msg_id && m.account_id == account_id);
}
pub(crate) fn copy_to_clipboard(text: &str) {
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
