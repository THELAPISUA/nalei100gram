use eframe::{
    egui::{Color32, Context, FontData, FontFamily, FontPriority, InsertFontFamily},
    epaint::text::FontInsert,
};

use crate::utils::types::Theme;

pub struct AppTheme {
    pub _background: Color32,
    pub surface: Color32,
    pub accent: Color32,
    pub text: Color32,
    pub muted: Color32,
}
impl From<Theme> for AppTheme {
    fn from(t: Theme) -> Self {
        Self {
            _background: Color32::from_rgb(t.background[0], t.background[1], t.background[2]),
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
            _background: Color32::from_rgb(18, 18, 26),
            surface: Color32::from_rgb(29, 29, 41),
            accent: Color32::from_rgb(145, 100, 255),
            text: Color32::from_rgb(240, 240, 250),
            muted: Color32::from_rgb(155, 155, 175),
        }
    }
}

pub fn setup_fonts(ctx: &Context) {
    let iosevka = FontInsert::new(
        "Iosevka",
        FontData::from_static(include_bytes!("../../assets/fonts/IosevkaAile-Bold.ttc")).into(),
        vec![
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Highest,
            },
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    );

    let emoji = FontInsert::new(
        "NotoColorEmoji",
        FontData::from_static(include_bytes!("../../assets/fonts/Noto-COLRv1.ttf")).into(),
        vec![
            InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            },
            InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Lowest,
            },
        ],
    );

    ctx.add_font(iosevka);
    ctx.add_font(emoji);
}
