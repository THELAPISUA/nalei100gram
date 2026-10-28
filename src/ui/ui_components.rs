use crate::ui::app::GramApp;

use crate::ui::logic::{copy_to_clipboard, delete_msg};
use crate::ui::widgets::{FileWidget, MessageAction, MessageWidget, PhotoWidget};

use crate::utils::types::{ActionPacket, MessagePacket};
use crate::utils::types::{Message, MsgType::*};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, vec2};
use egui_material_icons::MaterialIcon;
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
                    let mut actions = Vec::new();

                    if let Some(chat) = self
                        .chats
                        .iter_mut()
                        .find(|chat| chat.chat_id == self.current_chat)
                    {
                        for msg in &chat.messages {
                            if let Some(action) =
                                process_message(self.current_account_id, ui, msg, &mut self.photos)
                            {
                                actions.push(action);
                            }
                        }

                        chat.unreaded_count = 0;

                        for msg in &mut chat.messages {
                            msg.is_read = true;
                        }
                    }

                    for action in actions {
                        match action {
                            ActionPacket::Edit(message) => {
                                let _ = self.tx.try_send(ActionPacket::Edit(message));
                            }
                            ActionPacket::Delete(msg) => {
                                let chat_id = msg.chat_id;
                                let message_id = msg.id;

                                if let Some(chat) =
                                    self.chats.iter_mut().find(|chat| chat.chat_id == chat_id)
                                {
                                    delete_msg(message_id, chat);
                                }

                                let _ = self.tx.try_send(ActionPacket::Delete(msg));
                            }
                            _ => {}
                        }
                    }

                    preview_files_being_dropped(ui.ctx());
                });
        });
    }
}
//
// Короче я устав то давай, наклепай
// 1) Зроби обробку чатів, треба шукати серед чату конкретне сообщение
// 2) Зроби якусь удобну штуку для редактирования давай як в тг типа в поле ввода
// 3) Зроби видалення для себе і для всіх
// 4) Копіювання це просто в буфер кидай і якийсь плейсхолдер
fn process_message(
    account_id: i64,
    ui: &mut egui::Ui,
    msg: &MessagePacket,
    photos: &mut HashMap<String, egui::TextureHandle>,
) -> Option<ActionPacket> {
    match msg.msg_type {
        TEXT => {
            let widget = MessageWidget {
                username: msg.username.clone(),
                text: msg.text.clone(),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };

            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::_Edit => {
                        let id = msg.message_id?;

                        return Some(ActionPacket::Edit(Message {
                            id: id,
                            chat_id: msg.chat_id,
                            new_text: Some("hello".into()),
                            account_id,
                        }));
                    }
                    MessageAction::Copy => {
                        copy_to_clipboard(&msg.text);
                    }
                    MessageAction::Delete => {
                        let id = msg.message_id?;
                        return Some(ActionPacket::Delete(Message {
                            id: id,
                            chat_id: msg.chat_id,
                            new_text: None,
                            account_id,
                        }));
                    }
                }
            }
        }

        IMAGE | STICKER => {
            let widget = PhotoWidget {
                username: msg.username.clone(),
                texture: get_img(ui.ctx(), &msg.text, photos),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };
            widget.show(ui);
        }

        FILE => {
            let path = msg.text.clone();

            let widget = FileWidget {
                username: msg.username.clone(),
                _file_path: path.clone(),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
                file_name: msg.message_name.clone().unwrap_or("FILE".into()),
            };

            if widget.show(ui) {
                let _ = open::that(path);
            }
        }

        _ => {
            let widget = MessageWidget {
                username: msg.username.clone(),
                text: format!("Ошибка рендера неизвестний тип"),
                time: msg.time.clone(),
                outgoing: msg.is_bot,
            };

            widget.show(ui);
        }
    }

    None
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

fn load_texture(
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
pub(crate) fn draw_circle_button(
    ui: &mut egui::Ui,
    has_text: bool,
    icon: MaterialIcon,
) -> egui::Response {
    let (btn_color, icon_color) = {
        let visuals = ui.visuals();

        if has_text {
            (visuals.selection.bg_fill, visuals.selection.stroke.color)
        } else {
            (visuals.widgets.inactive.bg_fill, visuals.text_color())
        }
    };

    let (rect, response) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());

    ui.painter().circle_filled(rect.center(), 20.0, btn_color);

    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        String::from(icon),
        FontId::proportional(18.0),
        icon_color,
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    response
}
pub(crate) fn draw_avatar(
    ui: &mut egui::Ui,
    chat_id: i64,
    name: &str,
    avatar_center: Pos2,
    photos: &mut HashMap<String, egui::TextureHandle>,
) {
    let hue = (chat_id.unsigned_abs() % 360) as f32 / 360.0;
    let avatar_color: Color32 = egui::ecolor::Hsva::new(hue, 0.55, 0.80, 1.0).into();

    let painter = ui.painter();

    let path_str = format!("data/temp/{}.png", chat_id);
    let path = std::path::Path::new(&path_str);

    if path.exists() {
        // Фото должно быть заранее обрезано по кругу
        let size = 48.0;
        let avatar_rect = Rect::from_center_size(avatar_center, vec2(size, size));

        painter.image(
            get_img(ui.ctx(), &path_str, photos).id(),
            avatar_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.circle_filled(avatar_center, 24.0, avatar_color);

        painter.text(
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
    }
}
