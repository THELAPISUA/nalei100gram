use crate::logic::integrations::types::{ActionPacket, Chat};
use crate::ui::settings_widget::SettingsWidget;
use crate::ui::ui_components::{
    draw_avatar, draw_circle_button, get_img, preview_files_being_dropped,
};
use crate::utils::audio::AudioPlayer;
use crate::utils::types::{PendingFile, Settings};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, Sense, Stroke,
    TextEdit, vec2,
};
use egui_material_icons::icons::{ICON_SEND, ICON_SETTINGS};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};

pub struct GramApp {
    pub(crate) tx: Sender<ActionPacket>,
    pub(crate) rx: Receiver<ActionPacket>,
    pub(crate) chats: Vec<Chat>,
    pub(crate) input_text: String,
    pub(crate) current_chat: i64,
    pub(crate) accounts: Vec<(i64, String)>,
    pub(crate) current_account_id: i64,
    pub(crate) photos: HashMap<String, egui::TextureHandle>,
    pub(crate) pending_files: Vec<PendingFile>,
    pub(crate) settings_window: SettingsWidget,
    pub(crate) player: Arc<Mutex<AudioPlayer>>,
}

impl GramApp {
    pub fn new(
        tx: Sender<ActionPacket>,
        rx: Receiver<ActionPacket>,
        accounts: Vec<(i64, String)>,
        chats: Vec<Chat>,
        settings: Settings,
        player: Arc<Mutex<AudioPlayer>>,
    ) -> Self {
        Self {
            tx,
            rx,
            chats: chats,
            input_text: String::new(),
            current_chat: 0,
            photos: HashMap::new(),
            pending_files: Vec::new(),
            settings_window: SettingsWidget::default(settings),
            current_account_id: accounts.clone().first().unwrap().0,
            accounts,
            player,
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

        egui::Panel::left("my_left_panel")
            .min_size(250.0)
            .max_size(500.0)
            .show(ui, |ui| {
                ui.add_space(4.0);

                let chats = self.chats.clone();

                let mut sorted_chats: Vec<(i64, String)> = chats
                    .iter()
                    .map(|item| (item.chat_id, item.name.clone()))
                    .collect();

                sorted_chats.sort_by(|a, b| a.1.cmp(&b.1));

                let visuals = ui.visuals().clone();
                let row_height = 64.0;

                ui.spacing_mut().item_spacing.y = 0.0;

                for (chat_id, name) in sorted_chats {
                    let is_selected = self.current_chat == chat_id;

                    let unreaded = self
                        .chats
                        .iter()
                        .find(|chat| chat.chat_id == chat_id)
                        .map(|c| c.unreaded_count)
                        .unwrap_or(0);

                    let (rect, response) = ui.allocate_exact_size(
                        vec2(ui.available_width(), row_height),
                        Sense::click(),
                    );

                    let bg = if is_selected {
                        visuals.selection.bg_fill
                    } else if response.hovered() {
                        visuals.widgets.hovered.weak_bg_fill
                    } else {
                        Color32::TRANSPARENT
                    };

                    ui.painter().rect_filled(
                        rect.shrink2(vec2(4.0, 2.0)),
                        CornerRadius::same(10),
                        bg,
                    );

                    let (title_color, sub_color) = if is_selected {
                        (
                            visuals.selection.stroke.color,
                            visuals.selection.stroke.color,
                        )
                    } else {
                        (visuals.strong_text_color(), visuals.weak_text_color())
                    };

                    // Аватар
                    let avatar_center = rect.left_center() + vec2(36.0, 0.0);
                    draw_avatar(ui, chat_id, &name, avatar_center, &mut self.photos);

                    // Назва чату
                    let text_x = avatar_center.x + 36.0;

                    ui.painter().text(
                        egui::pos2(text_x, rect.top() + 14.0),
                        Align2::LEFT_TOP,
                        &name,
                        FontId::proportional(16.0),
                        title_color,
                    );

                    // Останнє повідомлення
                    let last_message = self.last_message(chat_id, self.current_account_id);

                    ui.painter().text(
                        egui::pos2(text_x, rect.top() + 36.0),
                        Align2::LEFT_TOP,
                        last_message,
                        FontId::proportional(13.0),
                        sub_color,
                    );

                    // Бейдж непрочитаних
                    if unreaded > 0 {
                        let text = unreaded.to_string();
                        let badge_w = (text.chars().count() as f32 * 8.0 + 14.0).max(22.0);

                        let badge_rect = egui::Rect::from_center_size(
                            egui::pos2(rect.right() - 14.0 - badge_w / 2.0, rect.bottom() - 20.0),
                            vec2(badge_w, 22.0),
                        );

                        let badge_bg = visuals.selection.bg_fill;
                        let badge_fg = visuals.selection.stroke.color;

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
                        log::info!("Clicked chat: {chat_id}");
                        self.current_chat = chat_id;
                    }

                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                }
                egui::Panel::bottom("dock_panel").show(ui, |ui| {
                    let name = self
                        .accounts
                        .iter()
                        .find(|a| a.0 == self.current_account_id)
                        .map(|n| n.1.clone())
                        .unwrap();

                    ui.horizontal(|ui| {
                        let path_str = format!("data/temp/{}.png", name);
                        let path = std::path::Path::new(&path_str);

                        let size = 48.0;
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

                        if path.exists() {
                            let ctx = ui.ctx();
                            let photo = get_img(ctx, &path_str, &mut self.photos);

                            ui.put(
                                rect,
                                egui::Image::new(&photo).fit_to_exact_size(rect.size()),
                            );
                        } else {
                            ui.painter().circle_filled(
                                rect.center(),
                                size / 2.0,
                                Color32::from_rgb(100, 100, 100),
                            );
                        }

                        egui::Popup::menu(&response).show(|ui| {
                            for (id, name) in &self.accounts {
                                if ui.button(name).clicked() && self.current_account_id != *id {
                                    self.current_account_id = *id;
                                }
                            }
                        });
                        ui.label(name);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if draw_circle_button(ui, true, ICON_SETTINGS).clicked() {
                                self.settings_window.open = true;
                            }
                        });
                    });
                });
            });
        egui::Panel::top("chat_dock").show(ui, |ui| {
            ui.horizontal(|ui| {
                let name = self
                    .chats
                    .iter()
                    .find(|chat| chat.chat_id == self.current_chat)
                    .map(|chat| chat.name.clone())
                    .unwrap_or_default();

                ui.heading(name)
            });
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
                    let input_w = ui.available_width() - 52.0 - 28.0;
                    let text_input_res = Frame::new()
                        .fill(ui.visuals().extreme_bg_color)
                        .stroke(Stroke::new(
                            1.0,
                            ui.visuals().widgets.noninteractive.bg_stroke.color,
                        ))
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(Margin::symmetric(14, 9))
                        .show(ui, |ui| {
                            ui.set_width(input_w.max(1.0));
                            ui.add(
                                TextEdit::singleline(&mut self.input_text)
                                    .frame(Frame::NONE)
                                    .hint_text("Написати повідомлення...")
                                    .desired_width(f32::INFINITY),
                            )
                        })
                        .inner;

                    let has_text = !self.input_text.trim().is_empty();
                    let send_res = draw_circle_button(ui, has_text, ICON_SEND);
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
        self.settings_window.show(ui.ctx());
    }
}
