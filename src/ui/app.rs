use crate::ui::ui_components::{draw_send_button, preview_files_being_dropped};
use crate::utils::types::{ActionPacket, Chat, MessagePacket, PendingFile};
use eframe::egui;
use eframe::egui::{
    Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Sense, Stroke, TextEdit,
    vec2,
};
use std::collections::HashMap;

use teloxide::types::ChatId;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct GramApp {
    pub(crate) tx: Sender<ActionPacket>,
    pub(crate) rx: Receiver<ActionPacket>,
    pub(crate) chats: Vec<Chat>,
    pub(crate) input_text: String,
    pub(crate) current_chat: ChatId,
    pub(crate) photos: HashMap<String, egui::TextureHandle>,
    pub(crate) pending_files: Vec<PendingFile>,
}

impl GramApp {
    pub fn new(tx: Sender<ActionPacket>, rx: Receiver<ActionPacket>, chats: Vec<Chat>) -> Self {
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
}

impl eframe::App for GramApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // блокировка закрития окна
        self.cancel_close(ui.ctx());
        // drug and drop
        self.collect_dropped_files(ui.ctx());
        // обработка drug and drop
        self.sender_docs_messages();

        //Тут получаем сообщения
        self.recv_messages();

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

                ui.painter().text(
                    egui::pos2(text_x, rect.top() + 36.0),
                    Align2::LEFT_TOP,
                    self.last_message(chat_id),
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

                    let has_text = !self.input_text.trim().is_empty();
                    let send_res = draw_send_button(ui, has_text);
                    let send_clicked = send_res.clicked();
                    let enter_pressed = text_input_res.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if (send_clicked || enter_pressed) && has_text {
                        self.send_message_packet(self.input_text.clone());
                        text_input_res.request_focus();
                    }
                });
            });
        //рендер откритого чата
        self.render_messages(ui);

        preview_files_being_dropped(ui.ctx());
    }
}
