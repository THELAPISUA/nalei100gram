use crate::ui::app::GramApp;
use crate::utils::types::ActionPacket::Send;
use crate::utils::types::MsgType::{FILE, IMAGE, STICKER, TEXT};
use crate::utils::types::{Chat, MessagePacket, PendingFile};
use arboard::Clipboard;
use chrono::Local;
use eframe::egui;
use log::info;
use std::ffi::OsStr;
use teloxide::types::{ChatId, MessageId};

impl GramApp {
    pub(crate) fn get_or_create_chat(&mut self, chat_id: ChatId, name: String) -> &mut Chat {
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
                _ => {}
            }
        }
    }
    pub(crate) fn sender_docs_messages(&mut self) {
        while let Some(file) = self.pending_files.pop() {
            //40 мб це ліміт на бота в тг
            if file.size <= 41943040 {
                let message_type = match file.extension.as_str() {
                    "png" | "jpg" | "webp" => IMAGE,
                    _ => FILE,
                };
                let msg = MessagePacket {
                    time: Local::now().format("%H:%M").to_string(),
                    username: String::from("You"),
                    text: file.path.to_string_lossy().to_string(),
                    chat_id: self.current_chat.clone(),
                    msg_type: message_type,
                    is_bot: true,
                    is_read: true,
                    message_name: Some(file.name),
                    message_id: None,

                    chat_name: None,
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
            .find(|c| c.chat_id == ChatId(chad_id))
            .map(|c| &c.messages);

        let default_msg = MessagePacket {
            username: String::new(),
            text: String::from("Поки нема повідомлень!"),
            ..Default::default()
        };
        let last_msg_p = curr_chat
            .and_then(|messages| messages.last())
            .unwrap_or(&default_msg);

        match last_msg_p.msg_type {
            TEXT => last_msg_p.text.clone(),
            IMAGE => String::from("Фото"),
            STICKER => String::from("Стикер"),
            FILE => String::from("ФАЙЛ"),
            _ => String::from("Невідомий формат"),
        }
    }
    pub(crate) fn send_message_packet(&mut self, msg: String) {
        let packet = MessagePacket {
            time: Local::now().format("%H:%M").to_string(),
            username: String::from("You"),
            text: msg.clone(),
            chat_id: self.current_chat.clone(),
            msg_type: TEXT,
            is_bot: true,
            is_read: true,
            message_name: None,
            message_id: None,

            ..Default::default()
        };
        if self.tx.try_send(Send(packet.clone())).is_ok() {
            let current_chat = self.current_chat.clone();
            let chat = self.get_or_create_chat(current_chat.clone(), String::new());

            chat.messages.push(packet);

            self.input_text.clear();
        }
    }
}

pub(crate) fn delete_msg(msg_id: MessageId, chat: &mut Chat) {
    chat.messages.retain(|m| m.message_id != Some(msg_id));
}
pub(crate) fn copy_to_clipboard(text: &str) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text("Привет из Rust!").unwrap();
}
