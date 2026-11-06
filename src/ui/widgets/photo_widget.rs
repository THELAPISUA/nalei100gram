use eframe::egui;

use crate::ui::widgets::types::MessageAction;
pub struct PhotoWidget {
    pub username: String,
    pub texture: egui::TextureHandle,
    pub time: String,
    pub outgoing: bool,
    pub caption: Option<String>,
}

impl PhotoWidget {
    const MAX_WIDTH: f32 = 380.0;
    const MAX_HEIGHT: f32 = 300.0;
    const MIN_WIDTH: f32 = 140.0;

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

    fn image_size(&self, max_width: f32) -> egui::Vec2 {
        let tex_size = self.texture.size_vec2();

        if tex_size.x <= 0.0 || tex_size.y <= 0.0 {
            return egui::vec2(Self::MIN_WIDTH, Self::MIN_WIDTH);
        }

        let max_w = max_width.min(Self::MAX_WIDTH);

        let scale = (max_w / tex_size.x)
            .min(Self::MAX_HEIGHT / tex_size.y)
            .min(1.0);

        tex_size * scale
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
        let text_color = visuals.text_color();

        let margin = egui::Margin {
            left: 8,
            right: 8,
            top: 6,
            bottom: 8,
        };
        let horizontal_margin = (margin.left + margin.right) as f32;

        let max_content_width = (ui.available_width() - horizontal_margin)
            .min(Self::MAX_WIDTH)
            .max(Self::MIN_WIDTH);

        let img_size = self.image_size(max_content_width);
        let content_width = img_size.x.max(Self::MIN_WIDTH);

        let frame_response = egui::Frame::new()
            .fill(bubble_color)
            .stroke(egui::Stroke::new(
                1.0,
                visuals.widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(margin)
            .show(ui, |ui| {
                ui.set_width(content_width);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&self.username)
                                .strong()
                                .size(14.0)
                                .color(username_color),
                        );
                    });

                    ui.add_space(4.0);

                    let image = egui::Image::from_texture(&self.texture)
                        .fit_to_exact_size(img_size)
                        .corner_radius(egui::CornerRadius::same(8));

                    ui.add(image);
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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&self.time)
                                    .size(11.0)
                                    .color(secondary_color),
                            );
                        });
                    });
                });
            })
            .response
            .interact(egui::Sense::click());

        self.popup(&frame_response)
    }

    fn popup(&self, response: &egui::Response) -> Option<MessageAction> {
        let mut action = None;

        egui::Popup::context_menu(response).show(|ui| {
            ui.set_min_width(140.0);

            if ui.button("Відкрити").clicked() {
                action = Some(MessageAction::Open);
                ui.close();
            }

            if ui.button("Копіювати").clicked() {
                action = Some(MessageAction::Copy);
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
