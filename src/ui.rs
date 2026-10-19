use crate::types::MsgType::{FILE, IMAGE, STICKER, TEXT};
use crate::types::{Chat, MessagePacket, MsgType, PendingFile};
use crate::widgets::{FileWidget, MessageWidget, PhotoWidget};
use chrono::Local;
use eframe::egui;

use eframe::egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Sense, Stroke, TextEdit,
    vec2,
};
use log::info;

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
    pending_files: Vec<PendingFile>,
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
            pending_files: Vec::new(),
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
        // блокировка закрития окна
        let ctx = ui.ctx();
        // днд
        collect_dropped_files(ctx, &mut self.pending_files);
        while let Some(file) = self.pending_files.pop() {
            let msg = MessagePacket {
                time: Local::now().format("%H:%M").to_string(),
                username: Some(String::from("You")),
                text: file.path.to_string_lossy().to_string(),
                chat_id: self.current_chat.clone(),
                msg_type: FILE,
                is_bot: true,
                is_read: true,
                msg_name: Some(file.name),
            };
            let is_ok = self.tx.try_send(msg.clone()).is_ok();
            if is_ok {
                let current_chat = self.current_chat.clone();
                let chat = self.get_or_create_chat(current_chat.clone());
                chat.messages.push(msg);
            }
        }

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
            ui.add_space(4.0);

            let chats = HashMap::from([(1714925468, "Кирилл"), (1136439451, "Влад")]);

            let mut sorted_chats: Vec<(&i64, &&str)> = chats.iter().collect();
            sorted_chats.sort_by_key(|&(_, name)| name);

            let telegram_blue = Color32::from_rgb(51, 144, 236);
            let row_height = 64.0;

            ui.spacing_mut().item_spacing.y = 0.0;

            for (&chat_id, &name) in sorted_chats {
                let is_selected = self.current_chat == ChatId(chat_id);

                let unreaded = self
                    .chats
                    .iter()
                    .find(|chat| chat.chat_id == ChatId(chat_id))
                    .map(|c| c.unreaded_count)
                    .unwrap_or(0);

                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_height), Sense::click());

                let bg = if is_selected {
                    telegram_blue
                } else if response.hovered() {
                    ui.visuals().widgets.hovered.weak_bg_fill
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter()
                    .rect_filled(rect.shrink2(vec2(4.0, 2.0)), CornerRadius::same(10), bg);

                let (title_color, sub_color) = if is_selected {
                    (Color32::WHITE, Color32::from_white_alpha(200))
                } else {
                    (
                        ui.visuals().strong_text_color(),
                        ui.visuals().weak_text_color(),
                    )
                };

                let avatar_center = rect.left_center() + vec2(12.0 + 24.0, 0.0);
                let hue = (chat_id.unsigned_abs() % 360) as f32 / 360.0;
                let avatar_color: Color32 = egui::ecolor::Hsva::new(hue, 0.55, 0.80, 1.0).into();
                ui.painter()
                    .circle_filled(avatar_center, 24.0, avatar_color);
                ui.painter().text(
                    avatar_center,
                    Align2::CENTER_CENTER,
                    name.chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string(),
                    FontId::proportional(20.0),
                    Color32::WHITE,
                );

                let text_x = avatar_center.x + 24.0 + 12.0;
                ui.painter().text(
                    egui::pos2(text_x, rect.top() + 14.0),
                    Align2::LEFT_TOP,
                    name,
                    FontId::proportional(16.0),
                    title_color,
                );
                // Так короче тут ми получаем останне сообщение от юзера
                // Короче винести треба в отдельну функцию
                let curr_chat = self
                    .chats
                    .iter()
                    .find(|c| c.chat_id == ChatId(chat_id))
                    .map(|c| &c.messages);

                let default_msg = MessagePacket {
                    username: Some(String::new()),
                    text: String::from("Поки нема повідомлень!"),
                    time: String::new(),
                    is_bot: false,
                    is_read: true,
                    msg_type: TEXT,
                    chat_id: ChatId(-1),
                    msg_name: None,
                };
                let last_msg_p = curr_chat
                    .and_then(|messages| messages.last())
                    .unwrap_or(&default_msg);

                let last_msg = match last_msg_p.msg_type {
                    TEXT => last_msg_p.text.clone(),
                    IMAGE => String::from("Фото"),
                    STICKER => String::from("Стикер"),
                    FILE => String::from("ФАЙЛ"),
                    _ => String::from("Невідомий формат"),
                };

                ui.painter().text(
                    egui::pos2(text_x, rect.top() + 36.0),
                    Align2::LEFT_TOP,
                    last_msg,
                    FontId::proportional(13.0),
                    sub_color,
                );

                // Бейдж непрочитаних
                if unreaded > 0 {
                    let text = unreaded.to_string();
                    let badge_w = (text.len() as f32 * 8.0 + 14.0).max(22.0);
                    let badge_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.right() - 14.0 - badge_w / 2.0, rect.bottom() - 20.0),
                        vec2(badge_w, 22.0),
                    );
                    let badge_bg = if is_selected {
                        Color32::WHITE
                    } else {
                        telegram_blue
                    };
                    let badge_fg = if is_selected {
                        telegram_blue
                    } else {
                        Color32::WHITE
                    };
                    ui.painter()
                        .rect_filled(badge_rect, CornerRadius::same(11), badge_bg);
                    ui.painter().text(
                        badge_rect.center(),
                        Align2::CENTER_CENTER,
                        text,
                        FontId::proportional(13.0),
                        badge_fg,
                    );
                }

                if response.clicked() {
                    self.current_chat = ChatId(chat_id);
                }
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        });
        //ввод сообщений
        egui::Panel::bottom("input_panel")
            .frame(
                Frame::new()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                let telegram_blue = Color32::from_rgb(51, 144, 236);

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    // Поле вводу в заокругленій рамці
                    let input_w = ui.available_width() - 52.0;
                    let text_input_res = Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .stroke(Stroke::new(
                            1.0,
                            ui.visuals().widgets.noninteractive.bg_stroke.color,
                        ))
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(Margin::symmetric(14, 9))
                        .show(ui, |ui| {
                            ui.set_width(input_w - 28.0);
                            ui.add(
                                TextEdit::singleline(&mut self.input_text)
                                    .frame(Frame::NONE)
                                    .hint_text("Написати повідомлення...")
                                    .desired_width(f32::INFINITY),
                            )
                        })
                        .inner;

                    // Кругла кнопка відправки
                    let has_text = !self.input_text.trim().is_empty();
                    let (rect, send_res) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
                    let btn_color = if has_text {
                        telegram_blue
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    ui.painter().circle_filled(rect.center(), 20.0, btn_color);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        ">",
                        FontId::proportional(18.0),
                        if has_text {
                            Color32::WHITE
                        } else {
                            ui.visuals().weak_text_color()
                        },
                    );
                    if send_res.hovered() && has_text {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    let send_clicked = send_res.clicked();
                    let enter_pressed = text_input_res.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (send_clicked || enter_pressed) && has_text {
                        let msg = self.input_text.clone();

                        if self
                            .tx
                            .try_send(MessagePacket {
                                time: Local::now().format("%H:%M").to_string(),
                                username: Some(String::from("You")),
                                text: msg.clone(),
                                chat_id: self.current_chat.clone(),
                                msg_type: TEXT,
                                is_bot: true,
                                is_read: true,
                                msg_name: None,
                            })
                            .is_ok()
                        {
                            let current_chat = self.current_chat.clone();
                            let chat = self.get_or_create_chat(current_chat.clone());
                            let msg = MessagePacket {
                                username: Some(String::from("You")),
                                text: msg,
                                is_bot: true,
                                time: Local::now().format("%H:%M").to_string(),
                                is_read: true,
                                msg_type: TEXT,
                                chat_id: current_chat,
                                msg_name: None,
                            };
                            chat.messages.push(msg.clone());

                            self.input_text.clear();
                        }

                        text_input_res.request_focus();
                    }
                });
            });
        //рендер откритого чата
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for chat in &mut self.chats {
                        if chat.chat_id == self.current_chat {
                            for msg in &chat.messages {
                                match msg.msg_type {
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
                                    FILE => {
                                        let path = msg.text.clone();
                                        let widget = FileWidget {
                                            username: msg
                                                .username
                                                .clone()
                                                .unwrap_or(String::from("You")),
                                            file_path: path.clone(),
                                            time: msg.time.clone(),
                                            outgoing: msg.is_bot,
                                            file_name: msg
                                                .msg_name
                                                .clone()
                                                .unwrap_or("FILE".to_string()),
                                        };
                                        if widget.show(ui) {
                                            let _ = open::that(path);
                                        }
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
            preview_files_being_dropped(ui.ctx());
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

pub fn collect_dropped_files(ctx: &egui::Context, out: &mut Vec<PendingFile>) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());

    for file in dropped {
        let path = file.path().to_path_buf();

        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        info!("collected file: {}", name);

        out.push(PendingFile { name, path, size });
    }
}

/// Затемнення з підказкою, поки файли тягнуть над вікном.
pub fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
    use std::fmt::Write as _;

    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }

    let text = ctx.input(|i| {
        let mut text = "Відпусти файли, щоб надіслати:\n".to_owned();
        for file in &i.raw.hovered_files {
            if let Some(path) = &file.path {
                write!(text, "\n{}", path.display()).unwrap();
            } else if !file.mime.is_empty() {
                write!(text, "\n{}", file.mime).unwrap();
            } else {
                text += "\n???";
            }
        }
        text
    });

    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
    let rect = ctx.content_rect();

    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(192));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(24.0),
        Color32::WHITE,
    );
}
