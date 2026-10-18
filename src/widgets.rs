use eframe::egui;

pub struct MessageWidget {
    pub username: String,
    pub text: String,
    pub time: String,
    pub outgoing: bool,
}

impl MessageWidget {
    pub fn show(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();

        ui.allocate_ui_with_layout(
            egui::vec2(available_width, 0.0),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                if self.outgoing {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        self.bubble(ui);
                    });
                } else {
                    self.bubble(ui);
                }
            },
        );

        ui.add_space(4.0);
    }

    fn bubble(&self, ui: &mut egui::Ui) {
        let bubble_color = if self.outgoing {
            egui::Color32::from_rgb(210, 240, 190)
        } else {
            egui::Color32::from_rgb(245, 245, 245)
        };

        egui::Frame::new()
            .fill(bubble_color)
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
                        .color(egui::Color32::from_rgb(40, 100, 180)),
                );

                ui.add_space(3.0);

                ui.label(egui::RichText::new(&self.text).size(15.0));

                ui.add_space(5.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&self.time)
                            .size(11.0)
                            .color(egui::Color32::GRAY),
                    );
                });
            });
    }
}

pub struct PhotoWidget {
    pub username: String,
    pub texture: egui::TextureHandle,
    pub time: String,
    pub outgoing: bool,
}
impl PhotoWidget {
    pub fn show(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();

        ui.allocate_ui_with_layout(
            egui::vec2(available_width, 0.0),
            egui::Layout::left_to_right(egui::Align::TOP),
            |ui| {
                if self.outgoing {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        self.bubble(ui);
                    });
                } else {
                    self.bubble(ui);
                }
            },
        );

        ui.add_space(4.0);
    }

    fn bubble(&self, ui: &mut egui::Ui) {
        let bubble_color = if self.outgoing {
            egui::Color32::from_rgb(210, 240, 190)
        } else {
            egui::Color32::from_rgb(245, 245, 245)
        };

        egui::Frame::new()
            .fill(bubble_color)
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin {
                left: 8,
                right: 8,
                top: 6,
                bottom: 8,
            })
            .show(ui, |ui| {
                // Фіксуємо ширину вмісту
                let content_width = 380.0;
                ui.set_width(content_width);

                ui.vertical(|ui| {
                    // Текст зверху
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&self.username)
                                .strong()
                                .size(14.0)
                                .color(egui::Color32::from_rgb(40, 100, 180)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&self.time)
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                            );
                        });
                    });

                    ui.add_space(4.0);

                    // Фото під текстом
                    let image = egui::Image::from_texture(&self.texture)
                        .max_width(content_width)
                        .max_height(300.0)
                        .corner_radius(egui::CornerRadius::same(8));

                    ui.add(image);
                });
            });
    }
}
