use eframe::egui;
use egui_material_icons::icons::{ICON_INFO, ICON_SETTINGS, ICON_WALLPAPER};

use crate::{
    ui::widgets::MessageWidget,
    utils::{
        settings::save_settings,
        themes::THEMES,
        types::{Settings, Theme},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Themes,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
}

pub struct SettingsWidget {
    pub open: bool,
    active_tab: SettingsTab,
    pub theme: AppTheme,
    pub selected_theme: usize,
    pub settings: Settings,
}

impl SettingsWidget {
    pub fn default(settings: Settings) -> Self {
        Self {
            open: false,
            active_tab: SettingsTab::General,
            theme: AppTheme::Dark,
            selected_theme: 0,
            settings,
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
                            format!("{} Загальні", ICON_SETTINGS.codepoint),
                        );

                        ui.selectable_value(
                            &mut self.active_tab,
                            SettingsTab::Themes,
                            format!("{} Теми", ICON_WALLPAPER.codepoint),
                        );

                        ui.selectable_value(
                            &mut self.active_tab,
                            SettingsTab::About,
                            format!("{} Про програму", ICON_INFO.codepoint),
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
                let theme = &THEMES[self.selected_theme].1;
                apply_theme(ctx, theme);
                self.settings.selected_theme = self.selected_theme;
                save_settings(self.settings.clone());
            }
        }

        if self.theme != previous_theme {
            let theme = &THEMES[self.selected_theme].1;
            apply_theme(ctx, theme);
            self.settings.selected_theme = self.selected_theme;
            save_settings(self.settings.clone());
        }

        ui.add_space(12.0);
        ui.separator();

        ui.label("Попередній перегляд");

        MessageWidget {
            username: "Вадим".to_string(),
            text: "О то усно можна не робить".to_string(),
            time: "3:00".to_string(),
            outgoing: false,
        }
        .show(ui);
        MessageWidget {
            username: "Костя".to_string(),
            text: "Не, усно це як письмово, тільки по другом".to_string(),
            time: "3:01".to_string(),
            outgoing: true,
        }
        .show(ui);
    }

    fn about_tab(&self, ui: &mut egui::Ui) {
        ui.heading("Про програму");
        ui.add_space(8.0);

        ui.label("Назва: Nalei100Gram");
        ui.label(format!("Версія: {}", env!("CARGO_PKG_VERSION")));
        ui.label("GUI: egui");
    }
}
pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = egui::Visuals::dark();

    let background = rgba(theme.background);
    let surface = rgba(theme.surface);
    let accent = rgba(theme.accent);
    // let text = rgba(theme.text);
    let muted = rgba(theme.muted);
    let border = rgba(theme.border);
    let hover = rgba(theme.hover);
    let input = rgba(theme.input);
    let accent_text = rgba(theme.accent_text);

    visuals.panel_fill = background;
    visuals.window_fill = surface;
    visuals.window_stroke = egui::Stroke::new(1.0, border);

    visuals.override_text_color = None;
    visuals.weak_text_color = Some(muted);
    visuals.hyperlink_color = accent;

    visuals.faint_bg_color = hover;
    visuals.extreme_bg_color = input;
    visuals.text_edit_bg_color = Some(input);
    visuals.code_bg_color = input;

    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = egui::Stroke::new(1.0, accent_text);

    visuals.warn_fg_color = egui::Color32::from_rgb(245, 180, 70);
    visuals.error_fg_color = egui::Color32::from_rgb(235, 80, 90);

    ctx.set_visuals(visuals);
}
fn rgba(color: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
}
