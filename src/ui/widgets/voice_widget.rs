use eframe::egui::{self, Align, Frame, Layout, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

use crate::ui::widgets::types::VoiceAction;

pub struct VoiceWidget {
    pub username: String,
    pub time: String,
    pub outgoing: bool,
    pub duration: String,
    pub progress: f64,
    pub playing: bool,
}

impl VoiceWidget {
    pub fn show(&mut self, ui: &mut Ui) -> VoiceAction {
        let visuals = ui.visuals().clone();

        let bubble_color = if self.outgoing {
            visuals.widgets.inactive.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };

        let mut action = VoiceAction::None;

        let layout = if self.outgoing {
            Layout::right_to_left(Align::Min)
        } else {
            Layout::left_to_right(Align::Min)
        };

        ui.with_layout(layout, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Username
                    ui.label(
                        RichText::new(&self.username)
                            .strong()
                            .size(12.0)
                            .color(ui.visuals().strong_text_color()),
                    );

                    // Voice message bubble
                    Frame::new()
                        .fill(bubble_color)
                        .stroke(Stroke::new(
                            1.0,
                            visuals.widgets.noninteractive.bg_stroke.color,
                        ))
                        .corner_radius(12.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.set_max_width(260.0);
                            ui.set_min_width(220.0);
                            action = self.bubble(ui);
                            ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                                ui.label(
                                    RichText::new(&self.time)
                                        .size(10.0)
                                        .color(ui.visuals().weak_text_color()),
                                );
                            });
                        });
                });
            });
        });

        action
    }

    fn bubble(&mut self, ui: &mut Ui) -> VoiceAction {
        let visuals = ui.visuals().clone();

        let bubble_color = if self.outgoing {
            visuals.widgets.inactive.weak_bg_fill
        } else {
            visuals.faint_bg_color
        };

        let accent = visuals.hyperlink_color;

        let mut action = VoiceAction::None;

        ui.horizontal(|ui| {
            // Play / Pause button
            let icon = if self.playing { "⏸" } else { "▶" };

            let button = egui::Button::new(RichText::new(icon).size(20.0).color(accent))
                .fill(bubble_color)
                .stroke(Stroke::NONE)
                .corner_radius(20.0);

            if ui.add_sized(Vec2::splat(36.0), button).clicked() {
                action = VoiceAction::PlayPause;
            }

            ui.vertical(|ui| {
                self.draw_waveform(ui, &mut action);

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.duration)
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                });
            });
        });

        action
    }

    fn draw_waveform(&mut self, ui: &mut Ui, action: &mut VoiceAction) {
        // Тимчасова форма хвилі
        const WAVEFORM: [f32; 35] = [
            0.3, 0.5, 0.8, 0.4, 0.6, 0.9, 0.5, 0.3, 0.7, 1.0, 0.6, 0.4, 0.8, 0.5, 0.3, 0.6, 0.9,
            0.7, 0.4, 0.8, 0.5, 0.3, 0.7, 0.9, 0.5, 0.4, 0.8, 0.6, 0.3, 0.5, 0.7, 0.9, 0.4, 0.6,
            0.5,
        ];

        let desired_size = Vec2::new(175.0, 34.0);

        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        let visuals = ui.visuals();

        let played_color = visuals.hyperlink_color;
        let remaining_color = visuals.weak_text_color();

        let progress = self.progress.clamp(0.0, 1.0);
        let played_width = rect.width() * progress as f32;

        let painter = ui.painter();

        let step = rect.width() / WAVEFORM.len() as f32;

        for (i, height) in WAVEFORM.iter().enumerate() {
            let x = rect.left() + i as f32 * step + step / 2.0;
            let bar_height = height * rect.height();

            let bar_rect = Rect::from_center_size(
                Pos2::new(x, rect.center().y),
                Vec2::new(3.0, bar_height.max(3.0)),
            );

            let color = if x <= rect.left() + played_width {
                played_color
            } else {
                remaining_color
            };

            painter.rect_filled(bar_rect, 2.0, color);
        }

        // Поточна позиція
        let cursor_x = rect.left() + played_width;

        painter.line_segment(
            [
                Pos2::new(cursor_x, rect.top()),
                Pos2::new(cursor_x, rect.bottom()),
            ],
            Stroke::new(1.5, played_color),
        );

        // Перемотування натисканням або перетягуванням
        if response.clicked() || response.dragged() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let progress = ((pointer.x - rect.left()) / rect.width()).clamp(0.0, 1.0) as f64;

                self.progress = progress;
                *action = VoiceAction::Seek(progress);
            }
        }
    }
}
