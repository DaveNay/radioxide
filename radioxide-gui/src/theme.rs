use iced::theme::Palette;
use iced::{Color, Theme};

// Main backgrounds
pub const PANEL_BG: Color = Color::from_rgb(0.102, 0.102, 0.102); // #1a1a1a
pub const PANEL_SURFACE: Color = Color::from_rgb(0.145, 0.145, 0.145); // #252525
pub const PANEL_BORDER: Color = Color::from_rgb(0.227, 0.227, 0.227); // #3a3a3a

// Text
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.878, 0.878, 0.878); // #e0e0e0
pub const TEXT_DIM: Color = Color::from_rgb(0.533, 0.533, 0.533); // #888888

// Frequency display
pub const FREQ_GREEN: Color = Color::from_rgb(0.0, 1.0, 0.533); // #00ff88
pub const FREQ_GLOW: Color = Color::from_rgba(0.0, 1.0, 0.533, 0.25); // #00ff88 @ 25%

// Accent
pub const ACCENT_BLUE: Color = Color::from_rgb(0.267, 0.533, 0.8); // #4488cc

// Buttons
pub const BUTTON_BG: Color = Color::from_rgb(0.165, 0.165, 0.165); // #2a2a2a
pub const BUTTON_HOVER: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333
pub const BUTTON_SELECTED: Color = Color::from_rgb(0.227, 0.353, 0.541); // #3a5a8a

// PTT / Tune
pub const PTT_RED: Color = Color::from_rgb(0.8, 0.2, 0.2); // #cc3333
pub const PTT_GREEN: Color = Color::from_rgb(0.2, 0.667, 0.2); // #33aa33
pub const TUNE_AMBER: Color = Color::from_rgb(0.8, 0.667, 0.2); // #ccaa33

// Status LEDs
pub const LED_GREEN: Color = Color::from_rgb(0.2, 0.8, 0.2); // #33cc33
pub const LED_RED: Color = Color::from_rgb(0.8, 0.2, 0.2); // #cc3333

// Sliders
pub const SLIDER_EMPTY: Color = Color::from_rgb(0.2, 0.2, 0.2); // #333333

pub fn radioxide_theme() -> Theme {
    Theme::custom(
        "Radioxide".into(),
        Palette {
            background: PANEL_BG,
            text: TEXT_PRIMARY,
            primary: ACCENT_BLUE,
            success: Color::from_rgb(0.2, 0.667, 0.2),
            danger: Color::from_rgb(0.8, 0.2, 0.2),
        },
    )
}
