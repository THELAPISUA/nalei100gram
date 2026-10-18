use crate::types::MsgType::{IMAGE, STICKER, TEXT};
use crate::types::{Chat, MessagePacket, MsgType};
use crate::widgets::{MessageWidget, PhotoWidget};
use chrono::Local;
use eframe::egui;

use std::collections::HashMap;

use image::ImageReader;

use std::io::Cursor;
use teloxide::types::ChatId;
use tokio::sync::mpsc::{Receiver, Sender};

const ERROR_LOAD: &[u8; 94191] = include_bytes!("../assets/error_load.png");

pub struct GramApp {
    tx: Sender<MessagePacket>,
    rx: Receiver<MessagePacket>,
    chats: Vec<Chat>,
    input_text: String,
    current_chat: ChatId,
    photos: HashMap<String, egui::TextureHandle>,
}

impl GramApp {
    pub fn new(tx: Sender<MessagePacket>, rx: Receiver<MessagePacket>, chats: Vec<Chat>) -> Self {
        Self {
            tx,
            rx,
            chats: chats,
            input_text: String::new(),
            current_chat: ChatId(0),
            photos: HashMap::new(),
        }
    }
    fn get_or_create_chat(&mut self, chat_id: ChatId) -> &mut Chat {
        if let Some(index) = self.chats.iter().position(|chat| chat.chat_id == chat_id) {
            return &mut self.chats[index];
        }

        self.chats.push(Chat {
            chat_id,
            messages: Vec::new(),
            unreaded_count: 0,
        });

        self.chats.last_mut().unwrap()
    }
}

impl eframe::App for GramApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        //Тут получаем сообщения
        while let Ok(packet) = self.rx.try_recv() {
            let chat_id = packet.chat_id;
            let chat = self.get_or_create_chat(chat_id);

            chat.unreaded_count += 1;
            chat.messages.push(packet.clone());
        }
        //Панель вибора чата
        ui.heading("Nalei100Gram");
        egui::Panel::left("my_left_panel").show(ui, |ui| {
            ui.label("ЧАТИ");

            let chats = HashMap::from([(1714925468, "Кирилл"), (1136439451, "Влад")]);

            let mut sorted_chats: Vec<(&i64, &&str)> = chats.iter().collect();
            sorted_chats.sort_by_key(|&(_, name)| name);

            ui.vertical(|ui| {
                for (&chat_id, &name) in sorted_chats {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.current_chat, ChatId(chat_id), name);

                        let chat = self
                            .chats
                            .iter()
                            .find(|chat| chat.chat_id == ChatId(chat_id));

                        let unreaded = match chat {
                            Some(c) => c.unreaded_count,
                            None => 0,
                        };

                        ui.label(format!("{}", unreaded));
                    });
                }
            });
        });
        //ввод сообщений
        egui::Panel::bottom("input_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                let text_input_res = ui.text_edit_singleline(&mut self.input_text);
                let send_clicked = ui.button("send").clicked();
                let enter_pressed =
                    text_input_res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if send_clicked || enter_pressed {
                    if self.input_text.is_empty() {
                        return;
                    }
                    let msg = self.input_text.clone();
                    if let Err(_err) = self.tx.try_send(MessagePacket {
                        time: Local::now().format("%H:%M").to_string(),
                        username: None,
                        text: msg.clone(),
                        chat_id: self.current_chat.clone(),
                        msgtype: TEXT,
                        is_bot: true,
                        is_read: true,
                    }) {
                    } else {
                        let current_chat = self.current_chat.clone();
                        let chat = self.get_or_create_chat(current_chat);
                        let message = MessagePacket {
                            username: Some(String::from("You")),
                            text: msg.clone(),
                            is_bot: true,
                            time: Local::now().format("%H:%M").to_string(),
                            is_read: true,
                            msgtype: MsgType::TEXT,
                            chat_id: current_chat,
                        };

                        chat.messages.push(message.clone());
                        self.input_text.clear();
                    }
                }
            });
        });
        //рендер откритого чата
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for chat in &mut self.chats {
                    if chat.chat_id == self.current_chat {
                        for msg in &chat.messages {
                            match msg.msgtype {
                                TEXT => {
                                    let widget = MessageWidget {
                                        username: msg
                                            .username
                                            .clone()
                                            .unwrap_or(String::from("You")),
                                        text: msg.text.clone(),
                                        time: msg.time.clone(),
                                        outgoing: msg.is_bot,
                                    };
                                    widget.show(ui);
                                }
                                IMAGE => {
                                    let widget = PhotoWidget {
                                        username: msg
                                            .username
                                            .clone()
                                            .unwrap_or(String::from("You")),
                                        texture: get_img(ui.ctx(), &msg.text, &mut self.photos),
                                        time: msg.time.clone(),
                                        outgoing: msg.is_bot,
                                    };
                                    widget.show(ui);
                                }
                                STICKER => {
                                    let widget = PhotoWidget {
                                        username: msg
                                            .username
                                            .clone()
                                            .unwrap_or(String::from("You")),
                                        texture: get_img(ui.ctx(), &msg.text, &mut self.photos),
                                        time: msg.time.clone(),
                                        outgoing: msg.is_bot,
                                    };
                                    widget.show(ui);
                                }
                                _ => {
                                    let widget = MessageWidget {
                                        username: msg
                                            .username
                                            .clone()
                                            .unwrap_or(String::from("You")),
                                        text: format!("Ошибка рендера неизвестний тип"),
                                        time: msg.time.clone(),
                                        outgoing: msg.is_bot,
                                    };
                                    widget.show(ui);
                                }
                            }
                        }
                        chat.unreaded_count = 0;
                        for msg in &mut chat.messages {
                            msg.is_read = true;
                        }

                        break;
                    }
                }
            });
        });
    }
}
pub fn get_img(
    ctx: &egui::Context,
    p: &String,
    photos: &mut HashMap<String, egui::TextureHandle>,
) -> egui::TextureHandle {
    if let Some(photo) = photos.get(p) {
        return photo.clone();
    }

    let data = std::fs::read(p.clone()).unwrap_or(ERROR_LOAD.to_vec());
    let photo_tmp = load_texture(ctx, p, &data).unwrap();
    photos.insert(p.clone(), photo_tmp.clone());
    photo_tmp
}

pub fn load_texture(
    ctx: &egui::Context,
    name: &str,
    data: &[u8],
) -> Result<egui::TextureHandle, image::ImageError> {
    let image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let image = image.to_rgba8();

    let size = [image.width() as usize, image.height() as usize];

    let pixels = image.as_flat_samples();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    Ok(ctx.load_texture(name, color_image, egui::TextureOptions::LINEAR))
}
