use crate::ui::app::GramApp;

use crate::ui::widgets::{FileWidget, MessageWidget, PhotoWidget};
use crate::utils::types::MessagePacket;
use crate::utils::types::MsgType::*;
use eframe::egui::{self, Align2, Color32, FontId, Sense, vec2};
use image::ImageReader;
use std::collections::HashMap;
use std::io::Cursor;

const ERROR_LOAD: &[u8; 94191] = include_bytes!("../../assets/error_load.png");

impl GramApp {
    pub(crate) fn render_messages(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for chat in &mut self.chats {
                        if chat.chat_id == self.current_chat {
                            for msg in &chat.messages {
                                process_message(ui, msg, &mut self.photos);
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

fn process_message(
    ui: &mut egui::Ui,
    msg: &MessagePacket,
    photos: &mut HashMap<String, egui::TextureHandle>,
) {
    match msg.msg_type {
        TEXT => {
            let widget = MessageWidget {
                username: msg.username.clone().unwrap_or(String::from("You")),
                text: msg.text.clone(),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };
            widget.show(ui);
        }
        IMAGE => {
            let widget = PhotoWidget {
                username: msg.username.clone().unwrap_or(String::from("You")),
                texture: get_img(ui.ctx(), &msg.text, photos),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };
            widget.show(ui);
        }
        STICKER => {
            let widget = PhotoWidget {
                username: msg.username.clone().unwrap_or(String::from("You")),
                texture: get_img(ui.ctx(), &msg.text, photos),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };
            widget.show(ui);
        }
        FILE => {
            let path = msg.text.clone();
            let widget = FileWidget {
                username: msg.username.clone().unwrap_or(String::from("You")),
                _file_path: path.clone(),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
                file_name: msg.msg_name.clone().unwrap_or("FILE".to_string()),
            };
            if widget.show(ui) {
                let _ = open::that(path);
            }
        }
        _ => {
            let widget = MessageWidget {
                username: msg.username.clone().unwrap_or(String::from("You")),
                text: format!("Ошибка рендера неизвестний тип"),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };
            widget.show(ui);
        }
    }
}

/// Затемнення з підказкою, поки файли тягнуть над вікном.
pub(crate) fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order};
    use std::fmt::Write as _;

    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }

    let text = ctx.input(|i| {
        let mut text = "Відпусти файли, щоб надіслати (40Мб max):\n".to_owned();
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
pub(crate) fn draw_send_button(ui: &mut egui::Ui, has_text: bool) -> egui::Response {
    let telegram_blue = Color32::from_rgb(51, 144, 236);
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
    send_res
}
