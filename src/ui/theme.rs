use eframe::{
    egui::{self, Color32, Context, FontData, FontDefinitions, FontFamily, Visuals},
    epaint::text::FontInsert,
};

use crate::utils::types::Theme;

pub struct AppTheme {
    pub background: Color32,
    pub surface: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub muted: Color32,
}
impl From<Theme> for AppTheme {
    fn from(t: Theme) -> Self {
        Self {
            background: Color32::from_rgb(t.background[0], t.background[1], t.background[2]),
            surface: Color32::from_rgb(t.surface[0], t.surface[1], t.surface[2]),
            accent: Color32::from_rgb(t.accent[0], t.accent[1], t.accent[2]),
            text: Color32::from_rgb(t.text[0], t.text[1], t.text[2]),
            muted: Color32::from_rgb(t.muted[0], t.muted[1], t.muted[2]),
        }
    }
}
impl Default for AppTheme {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(18, 18, 26),
            surface: Color32::from_rgb(29, 29, 41),
            accent: Color32::from_rgb(145, 100, 255),
            text: Color32::from_rgb(240, 240, 250),
            muted: Color32::from_rgb(155, 155, 175),
        }
    }
}

impl AppTheme {
    pub fn apply(&self, ctx: &Context) {
        let mut visuals = Visuals::dark();

        visuals.panel_fill = self.background;
        visuals.window_fill = self.surface;
        visuals.extreme_bg_color = Color32::from_rgb(14, 14, 20);
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(115, 75, 210);
        visuals.selection.bg_fill = self.accent;
        visuals.override_text_color = Some(self.text);

        ctx.set_visuals(visuals);
    }
}

pub fn setup_fonts(ctx: &Context) {
    let font = FontInsert::new(
        "Iosevka",
        FontData::from_static(include_bytes!("../../assets/fonts/IosevkaAile-Bold.ttc")).into(),
        vec![
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    );

    ctx.add_font(font);
}
