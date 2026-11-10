use eframe::egui::{self, Response, Sense};

use crate::ui::{logic::line_break, widgets::types::MessageAction};

pub struct MessageWidget {
    pub username: String,
    pub text: String,
    pub time: String,
    pub outgoing: bool,
}

impl MessageWidget {
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
        let visuals = ui.visuals().clone();

        let bubble_color = if self.outgoing {
            visuals.widgets.inactive.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };

        let username_color = visuals.hyperlink_color;
        let secondary_color = visuals.weak_text_color();

        let frame_response = egui::Frame::new()
            .fill(bubble_color)
            .stroke(egui::Stroke::new(
                1.0,
                visuals.widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin {
                left: 12,
                right: 12,
                top: 8,
                bottom: 6,
            })
            .show(ui, |ui| {
                ui.set_max_width(400.0);

                ui.label(
                    egui::RichText::new(&self.username)
                        .strong()
                        .size(14.0)
                        .color(username_color),
                );

                ui.add_space(3.0);

                ui.label(egui::RichText::new(line_break(&self.text)).size(15.0));

                ui.add_space(5.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&self.time)
                            .size(11.0)
                            .color(secondary_color),
                    );
                });
            })
            .response
            .interact(Sense::click());

        self.popup(&frame_response)
    }

    fn popup(&self, response: &Response) -> Option<MessageAction> {
        let mut action = None;

        egui::Popup::context_menu(response).show(|ui| {
            ui.set_min_width(140.0);

            if ui.button("Копіювати").clicked() {
                ui.ctx().copy_text(self.text.clone());
                action = Some(MessageAction::Copy);
                ui.close();
            }
            if self.outgoing {
                if ui.button("Редагувати").clicked() {
                    action = Some(MessageAction::Edit);
                    ui.close();
                }
            }

            if ui.button("Видалити").clicked() {
                action = Some(MessageAction::Delete);
                ui.close();
            }
        });

        action
    }
}
