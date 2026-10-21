use eframe::egui::{Color32, Context, Visuals};

pub struct AppTheme {
    pub background: Color32,
    pub surface: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub muted: Color32,
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
