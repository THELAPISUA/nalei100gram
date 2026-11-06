use eframe::egui::{self, Response};

use crate::ui::widgets::types::MessageAction;

pub struct FileWidget {
    pub username: String,
    pub time: String,
    pub file_name: String,
    pub _file_path: String,
    pub outgoing: bool,
    pub caption: Option<String>,
}

impl FileWidget {
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
        let click_id = egui::Id::new("file_bubble").with(ui.next_auto_id());
        let visuals = ui.visuals().clone();

        let bubble_color = if self.outgoing {
            visuals.widgets.inactive.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };

        let accent = visuals.hyperlink_color;
        let accent_hover = visuals.widgets.hovered.fg_stroke.color;
        let secondary_color = visuals.weak_text_color();
        let text_color = visuals.text_color();

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
                    ui.label(
                        egui::RichText::new(&self.username)
                            .strong()
                            .size(14.0)
                            .color(accent),
                    );

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());

                        let circle_color = if ui.rect_contains_pointer(rect) {
                            accent_hover
                        } else {
                            accent
                        };

                        let painter = ui.painter();
                        let c = rect.center();

                        painter.circle_filled(c, 24.0, circle_color);

                        // Іконка файлу
                        let page_color = visuals.panel_fill;
                        let fold_color = visuals.widgets.inactive.bg_fill;

                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                c + egui::vec2(-9.0, -12.0),
                                c + egui::vec2(2.0, -12.0),
                                c + egui::vec2(9.0, -5.0),
                                c + egui::vec2(9.0, 12.0),
                                c + egui::vec2(-9.0, 12.0),
                            ],
                            page_color,
                            egui::Stroke::NONE,
                        ));

                        painter.add(egui::Shape::convex_polygon(
                            vec![
                                c + egui::vec2(2.0, -12.0),
                                c + egui::vec2(2.0, -5.0),
                                c + egui::vec2(9.0, -5.0),
                            ],
                            fold_color,
                            egui::Stroke::NONE,
                        ));

                        let line = egui::Stroke::new(1.5, circle_color);

                        painter.line_segment(
                            [c + egui::vec2(-5.0, 1.0), c + egui::vec2(5.0, 1.0)],
                            line,
                        );

                        painter.line_segment(
                            [c + egui::vec2(-5.0, 5.0), c + egui::vec2(5.0, 5.0)],
                            line,
                        );

                        painter.line_segment(
                            [c + egui::vec2(-5.0, 9.0), c + egui::vec2(1.0, 9.0)],
                            line,
                        );

                        ui.add_space(4.0);

                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&self.file_name).strong().size(15.0),
                                )
                                .truncate(),
                            );
                        });
                    });

                    ui.horizontal(|ui| {
                        if let Some(caption) =
                            self.caption.as_deref().filter(|c| !c.trim().is_empty())
                        {
                            ui.add_space(6.0);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(caption).size(14.0).color(text_color),
                                )
                                .wrap(),
                            );
                        }
                        // Message time
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
