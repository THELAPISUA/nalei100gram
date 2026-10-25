use eframe::egui;

use crate::{ui::widgets::MessageWidget, utils::themes::THEMES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Themes,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

pub struct SettingsWidget {
    pub open: bool,
    active_tab: SettingsTab,
    pub theme: AppTheme,
    pub selected_theme: usize,
    pub notifications: bool,
    pub save_history: bool,
}

impl Default for SettingsWidget {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: SettingsTab::General,
            theme: AppTheme::Dark,
            selected_theme: 0,
            notifications: true,
            save_history: true,
        }
    }
}

impl SettingsWidget {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        let mut open = self.open;

        egui::Window::new("Налаштування")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([520.0, 360.0])
            .min_size([420.0, 300.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(130.0);

                        ui.selectable_value(
                            &mut self.active_tab,
                            SettingsTab::General,
                            "⚙ Загальні",
                        );

                        ui.selectable_value(&mut self.active_tab, SettingsTab::Themes, "🎨 Теми");

                        ui.selectable_value(
                            &mut self.active_tab,
                            SettingsTab::About,
                            "ⓘ Про програму",
                        );
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.set_min_width(260.0);

                        match self.active_tab {
                            SettingsTab::General => self.general_tab(ui),
                            SettingsTab::Themes => self.themes_tab(ui, ctx),
                            SettingsTab::About => self.about_tab(ui),
                        }
                    });
                });
            });

        self.open = open;
    }

    fn general_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Загальні налаштування");
        ui.add_space(8.0);
        ui.label("Поки нема:)");

        // ui.checkbox(&mut self.notifications, "Увімкнути сповіщення");

        // ui.checkbox(&mut self.save_history, "Зберігати історію повідомлень");
    }

    fn themes_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Оформлення");
        ui.add_space(8.0);

        ui.label("Виберіть тему програми:");

        let previous_theme = self.theme;

        for (index, (name, _theme)) in THEMES.iter().enumerate() {
            if ui
                .radio_value(&mut self.selected_theme, index, *name)
                .changed()
            {
                self.apply_theme(ctx);
            }
        }

        if self.theme != previous_theme {
            self.apply_theme(ctx);
        }

        ui.add_space(12.0);
        ui.separator();

        ui.label("Попередній перегляд");

        MessageWidget {
            username: "Яна Цист".to_string(),
            text: "Привет вась".to_string(),
            time: "3:00".to_string(),
            outgoing: false,
        }
        .show(ui);
    }

    fn about_tab(&self, ui: &mut egui::Ui) {
        ui.heading("Про програму");
        ui.add_space(8.0);

        ui.label("Назва: Nalei100Gram");
        ui.label("Версія: 0.7b");
        ui.label("GUI: egui");
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        let theme = &THEMES[self.selected_theme].1;

        let mut visuals = egui::Visuals::dark();

        visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(
            theme.background[0],
            theme.background[1],
            theme.background[2],
            theme.background[3],
        );

        visuals.window_fill = egui::Color32::from_rgba_unmultiplied(
            theme.surface[0],
            theme.surface[1],
            theme.surface[2],
            theme.surface[3],
        );

        visuals.override_text_color = Some(egui::Color32::from_rgba_unmultiplied(
            theme.text[0],
            theme.text[1],
            theme.text[2],
            theme.text[3],
        ));

        visuals.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(
            theme.accent[0],
            theme.accent[1],
            theme.accent[2],
            theme.accent[3],
        );

        ctx.set_visuals(visuals);
    }
}
