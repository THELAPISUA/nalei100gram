use eframe::egui::{self, Response};

use crate::ui::widgets::types::MessageAction;

pub struct AudioWidget {
    pub username: String,
    pub time: String,
    pub file_name: String,
    pub duration: String,
    pub outgoing: bool,

    pub progress: f64,
    pub playing: bool,
}

impl AudioWidget {
    /// Повертає true, якщо користувач натиснув на бульбашку.
    pub fn show(&self, ui: &mut egui::Ui) -> Option<MessageAction> {
        let available_width = ui.available_width();

        let inner = ui.allocate_ui_with_layout(
            egui::vec2(available_width, 0.0),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                if self.outgoing {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        self.bubble(ui)
                    })
                    .inner
                } else {
                    self.bubble(ui)
                }
            },
        );

        ui.add_space(4.0);
        inner.inner
    }

    fn bubble(&self, ui: &mut egui::Ui) -> Option<MessageAction> {
        let click_id = egui::Id::new("audio_bubble").with(ui.next_auto_id());

        let visuals = ui.visuals().clone();

        let bubble_color = if self.outgoing {
            visuals.widgets.inactive.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };

        let accent = visuals.hyperlink_color;
        let secondary_color = visuals.weak_text_color();

        let frame_response = egui::Frame::new()
            .fill(bubble_color)
            .stroke(egui::Stroke::new(
                1.0,
                visuals.widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin {
                left: 10,
                right: 10,
                top: 6,
                bottom: 6,
            })
            .show(ui, |ui| {
                ui.set_width(300.0);

                ui.vertical(|ui| {
                    // Username
                    ui.label(
                        egui::RichText::new(&self.username)
                            .strong()
                            .size(14.0)
                            .color(accent),
                    );

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        // Play / Pause button
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(42.0, 42.0), egui::Sense::click());

                        let button_color = if response.hovered() {
                            visuals.widgets.hovered.bg_fill
                        } else {
                            accent
                        };

                        let painter = ui.painter();

                        painter.circle_filled(rect.center(), 21.0, button_color);

                        let c = rect.center();

                        if self.playing {
                            // Pause icon
                            painter.rect_filled(
                                egui::Rect::from_center_size(
                                    c + egui::vec2(-4.0, 0.0),
                                    egui::vec2(3.0, 13.0),
                                ),
                                1.0,
                                visuals.panel_fill,
                            );

                            painter.rect_filled(
                                egui::Rect::from_center_size(
                                    c + egui::vec2(4.0, 0.0),
                                    egui::vec2(3.0, 13.0),
                                ),
                                1.0,
                                visuals.panel_fill,
                            );
                        } else {
                            // Play icon
                            painter.add(egui::Shape::convex_polygon(
                                vec![
                                    c + egui::vec2(-4.0, -8.0),
                                    c + egui::vec2(-4.0, 8.0),
                                    c + egui::vec2(8.0, 0.0),
                                ],
                                visuals.panel_fill,
                                egui::Stroke::NONE,
                            ));
                        }

                        // Audio information
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.file_name).strong().size(14.0),
                                )
                                .truncate(),
                            );

                            ui.add_space(4.0);

                            // Progress bar
                            let progress_width = ui.available_width();

                            ui.add(
                                egui::ProgressBar::new(self.progress.clamp(0.0, 1.0) as f32)
                                    .desired_width(progress_width)
                                    .desired_height(4.0)
                                    .corner_radius(egui::CornerRadius::same(2)),
                            );

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&self.duration)
                                        .size(11.0)
                                        .color(secondary_color),
                                );
                            });
                        });
                    });

                    ui.add_space(4.0);

                    // Message time
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&self.time)
                                    .size(11.0)
                                    .color(secondary_color),
                            );
                        });
                    });
                });
            });

        let response = ui
            .interact(frame_response.response.rect, click_id, egui::Sense::click())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        // ЛКМ
        if response.clicked_by(egui::PointerButton::Primary) {
            return Some(MessageAction::Clicked);
        }

        // ПКМ
        self.popup(&response)
    }
    fn popup(&self, response: &Response) -> Option<MessageAction> {
        let mut action = None;

        egui::Popup::context_menu(response).show(|ui| {
            ui.set_min_width(140.0);

            if ui.button("Відкрити").clicked() {
                action = Some(MessageAction::Open);
                ui.close();
            }

            if ui.button("Видалити").clicked() {
                action = Some(MessageAction::Delete);
                ui.close();
            }
        });

        action
    }
}
