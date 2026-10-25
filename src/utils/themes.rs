use crate::utils::types::Theme;

pub const THEMES: [(&str, Theme); 7] = [
    (
        "Default",
        Theme {
            background: [27, 27, 27, 255],
            surface: [40, 40, 40, 255],
            accent: [80, 100, 180, 255],
            text: [240, 240, 240, 255],
            muted: [160, 160, 160, 255],
        },
    ),
    (
        "Midnight",
        Theme {
            background: [18, 18, 27, 255],
            surface: [29, 30, 43, 255],
            accent: [130, 100, 255, 255],
            text: [235, 236, 245, 255],
            muted: [145, 148, 170, 255],
        },
    ),
    (
        "Ocean",
        Theme {
            background: [13, 25, 38, 255],
            surface: [23, 43, 60, 255],
            accent: [54, 184, 210, 255],
            text: [225, 241, 248, 255],
            muted: [135, 165, 180, 255],
        },
    ),
    (
        "Forest",
        Theme {
            background: [18, 28, 23, 255],
            surface: [29, 44, 35, 255],
            accent: [102, 190, 128, 255],
            text: [231, 242, 232, 255],
            muted: [145, 170, 150, 255],
        },
    ),
    (
        "Dracula",
        Theme {
            background: [40, 42, 54, 255],
            surface: [52, 55, 70, 255],
            accent: [189, 147, 249, 255],
            text: [248, 248, 242, 255],
            muted: [160, 161, 177, 255],
        },
    ),
    (
        "Nord",
        Theme {
            background: [46, 52, 64, 255],
            surface: [59, 66, 82, 255],
            accent: [136, 192, 208, 255],
            text: [236, 239, 244, 255],
            muted: [163, 174, 190, 255],
        },
    ),
    (
        "Light",
        Theme {
            background: [245, 247, 250, 255],
            surface: [255, 255, 255, 255],
            accent: [75, 105, 220, 255],
            text: [35, 40, 52, 255],
            muted: [115, 123, 140, 255],
        },
    ),
];
