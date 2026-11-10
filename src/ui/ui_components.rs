use crate::logic::integrations::types::MessageType::TEXT;
use crate::ui::app::{Action, GramApp};

use crate::logic::integrations::types::{ActionPacket, MessageType, ReceivedMessage};

use crate::ui::widgets::audio_widget::AudioWidget;
use crate::ui::widgets::file_widget::FileWidget;
use crate::ui::widgets::message_widget::MessageWidget;
use crate::ui::widgets::photo_widget::PhotoWidget;
use crate::ui::widgets::types::{MessageAction, VoiceAction};
use crate::ui::widgets::voice_widget::VoiceWidget;
use crate::utils::audio::AudioPlayer;

use crate::ui::logic::{
    copy_photo_to_clipboard, copy_text_to_clipboard, delete_msg, get_message_by_id,
};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, vec2};

use egui_material_icons::MaterialIcon;
use egui_material_icons::icons::ICON_BOOKMARK;
use image::ImageReader;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
                            if msg.account_id == self.current_account_id || msg.account_id == -1 {
                                if let Some(action) = process_message(
                                    self.current_account_id,
                                    ui,
                                    msg,
                                    &mut self.photos,
                                    &self.player,
                                ) {
                                    actions.push(action);
                                }
                            }
                        }

                        chat.unreaded_count = 0;

                        for msg in &mut chat.messages {
                            msg.readed = true;
                        }
                    }

                    for action in actions {
                        match action {
                            ActionPacket::Edit {
                                accaunt_id,
                                chat_id,
                                id,
                                new_text: _,
                            } => {
                                if let Some(chat) =
                                    self.chats.iter_mut().find(|chat| chat.chat_id == chat_id)
                                {
                                    let old_text =
                                        match get_message_by_id(id, accaunt_id, chat).unwrap().t {
                                            TEXT { ref text } => text.clone(),
                                            _ => String::new(),
                                        };
                                    self.input_text = old_text.clone();
                                    self.curr_action = Some(Action::Edit { id, old_text });
                                }
                            }
                            ActionPacket::Delete {
                                id,
                                account_id,
                                chat_id,
                            } => {
                                if let Some(chat) =
                                    self.chats.iter_mut().find(|chat| chat.chat_id == chat_id)
                                {
                                    delete_msg(id, account_id, chat, &self.tx);
                                }
                            }
                            _ => {}
                        }
                    }
                });
        });
    }
}
fn process_message(
    account_id: i64,
    ui: &mut egui::Ui,
    msg: &ReceivedMessage,
    photos: &mut HashMap<String, egui::TextureHandle>,
    player: &Arc<Mutex<AudioPlayer>>,
) -> Option<ActionPacket> {
    ui.ctx().request_repaint_after(Duration::from_millis(16));
    match msg.t {
        MessageType::TEXT { ref text } => {
            let widget = MessageWidget {
                username: msg.username.clone(),
                text: text.clone(),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
            };

            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::Edit => {
                        return Some(ActionPacket::Edit {
                            accaunt_id: account_id,
                            chat_id: msg.chat_id,
                            id: msg.id,
                            new_text: "".to_string(),
                        });
                    }
                    MessageAction::Copy => {
                        copy_text_to_clipboard(&text);
                    }
                    MessageAction::Delete => {
                        return Some(ActionPacket::Delete {
                            id: msg.id,
                            account_id,
                            chat_id: msg.chat_id,
                        });
                    }
                    _ => return None,
                }
            }
        }

        MessageType::PHOTO {
            ref path,
            ref caption,
        } => {
            let widget = PhotoWidget {
                username: msg.username.clone(),
                texture: get_img(ui.ctx(), &path, photos),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
                caption: caption.clone(),
            };

            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::Open => {
                        let _ = open::that(path);
                    }
                    MessageAction::Copy => copy_photo_to_clipboard(path),
                    MessageAction::Delete => {
                        return Some(ActionPacket::Delete {
                            id: msg.id,
                            account_id,
                            chat_id: msg.chat_id,
                        });
                    }
                    _ => return None,
                }
            }
        }

        MessageType::STICKER { ref path } => {
            let widget = PhotoWidget {
                username: msg.username.clone(),
                texture: get_img(ui.ctx(), &path, photos),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
                caption: None,
            };
            widget.show(ui);
            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::Delete => {
                        return Some(ActionPacket::Delete {
                            id: msg.id,
                            account_id,
                            chat_id: msg.chat_id,
                        });
                    }
                    _ => return None,
                }
            }
        }

        MessageType::FILE {
            ref path,
            ref name,
            ref caption,
        } => {
            let widget = FileWidget {
                username: msg.username.clone(),
                _file_path: path.clone(),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
                file_name: name.clone(),
                caption: caption.clone(),
            };
            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::Open | MessageAction::Clicked => {
                        let _ = open::that(path);
                    }
                    MessageAction::Delete => {
                        return Some(ActionPacket::Delete {
                            id: msg.id,
                            account_id,
                            chat_id: msg.chat_id,
                        });
                    }
                    _ => return None,
                }
            }
        }
        MessageType::AUDIO {
            ref path,
            ref name,
            ref caption,
        } => {
            let mut p = player.lock().unwrap();
            p.tick();
            let (position, playing, duration) = match p.get_state(path) {
                Some(state) => (state.position, state.playing, state.duration),
                None => (Duration::ZERO, false, Duration::ZERO),
            };
            let total = duration.as_secs_f64();
            let progress = if total > 0.0 {
                (position.as_secs_f64() / total).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let widget = AudioWidget {
                username: msg.username.clone(),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
                file_name: name.clone(),
                duration: format_time(position),
                progress,
                playing,
                caption: caption.clone(),
            };
            if let Some(action) = widget.show(ui) {
                match action {
                    MessageAction::Open => {
                        let _ = open::that(path);
                    }
                    MessageAction::Clicked => {
                        if playing {
                            p.toggle();
                        } else {
                            p.play_sound(path);
                        }
                    }
                    MessageAction::Delete => {
                        return Some(ActionPacket::Delete {
                            id: msg.id,
                            account_id,
                            chat_id: msg.chat_id,
                        });
                    }
                    _ => return None,
                }
            }

            if playing {
                ui.ctx().request_repaint_after(Duration::from_millis(50));
            }
        }
        MessageType::VOICE { ref path } => {
            let mut p = player.lock().unwrap();
            p.tick();
            let (position, playing, duration) = match p.get_state(path) {
                Some(state) => (state.position, state.playing, state.duration),
                None => (Duration::ZERO, false, Duration::ZERO),
            };

            let total = duration.as_secs_f64();
            let progress = if total > 0.0 {
                (position.as_secs_f64() / total).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let mut widget = VoiceWidget {
                username: msg.username.clone(),
                time: msg.date.clone(),
                outgoing: msg.sender_is_you,
                duration: format_time(position),
                progress,
                playing,
            };

            match widget.show(ui) {
                VoiceAction::PlayPause => {
                    if playing {
                        p.toggle();
                    } else {
                        p.play_sound(path);
                    }
                }
                VoiceAction::Seek(progress) => {
                    // перемотувати можна лише завантажений трек
                    if total > 0.0 {
                        p.seek(duration.mul_f64(progress));
                    }
                }
                VoiceAction::None => {}
            }

            if playing {
                ui.ctx().request_repaint_after(Duration::from_millis(50));
            }
        } // _ => {
          //     let widget = MessageWidget {
          //         username: msg.username.clone(),
          //         text: format!("Ошибка рендера неизвестний тип"),
          //         time: msg.date.clone(),
          //         outgoing: msg.sender_is_you,
          //     };

          //     widget.show(ui);
          // }
    }

    None
}

/// Затемнення з підказкою, поки файли тягнуть над вікном.
pub(crate) fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::{Align2, Color32, Id, LayerId, Order};
    use std::fmt::Write as _;

    let mut fit_into = true;

    if ctx.input(|i| i.raw.hovered_files.is_empty()) {
        return;
    }

    let text = ctx.input(|i| {
        let mut text = "Відпусти файли, щоб надіслати:\n".to_owned();
        for file in &i.raw.hovered_files {
            if let Some(path) = &file.path {
                fit_into = path.metadata().unwrap().len() / 1024 / 1024 <= 40;
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
    if fit_into {
        painter.rect_filled(rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(24.0),
            Color32::WHITE,
        );
    } else {
        painter.rect_filled(rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Файл завеликий (40Мб max)",
            egui::FontId::proportional(24.0),
            Color32::RED,
        );
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
    size: f32,
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

    ui.painter().circle_filled(rect.center(), size, btn_color);

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

        let name = if chat_id == 0 {
            String::from(ICON_BOOKMARK)
        } else {
            name.chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string()
        };
        painter.text(
            avatar_center,
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(20.0),
            Color32::WHITE,
        );
    }
}
fn format_time(duration: Duration) -> String {
    let secs = duration.as_secs();
    format!("{}:{:02}", secs / 60, secs % 60)
}
