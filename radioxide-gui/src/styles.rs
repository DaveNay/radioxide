use iced::widget::{button, container, slider, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

use crate::theme::*;

// --- Container styles ---

pub fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: Some(Background::Color(PANEL_SURFACE)),
        border: Border {
            color: PANEL_BORDER,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn freq_display_panel(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: Some(Background::Color(Color::BLACK)),
        border: Border {
            color: PANEL_BORDER,
            width: 2.0,
            radius: 6.0.into(),
        },
        shadow: Shadow {
            color: FREQ_GLOW,
            offset: Vector::ZERO,
            blur_radius: 12.0,
        },
    }
}

pub fn status_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: None,
        background: Some(Background::Color(PANEL_BG)),
        border: Border {
            color: PANEL_BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
    }
}

// --- Button styles ---

pub fn radio_button(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg = if is_selected {
            BUTTON_SELECTED
        } else {
            match status {
                button::Status::Hovered => BUTTON_HOVER,
                button::Status::Pressed => PANEL_BG,
                _ => BUTTON_BG,
            }
        };

        let text_color = if is_selected {
            Color::WHITE
        } else {
            TEXT_PRIMARY
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: Border {
                color: if is_selected {
                    ACCENT_BLUE
                } else {
                    PANEL_BORDER
                },
                width: 1.0,
                radius: 3.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn ptt_button(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = if is_active { PTT_RED } else { PTT_GREEN };
        let bg = match status {
            button::Status::Hovered => lighten(base, 0.15),
            button::Status::Pressed => darken(base, 0.15),
            _ => base,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::WHITE,
            border: Border {
                color: lighten(base, 0.2),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn tune_button(is_tuning: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let base = if is_tuning { TUNE_AMBER } else { BUTTON_BG };
        let bg = match status {
            button::Status::Hovered => lighten(base, 0.1),
            button::Status::Pressed => darken(base, 0.1),
            _ => base,
        };

        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if is_tuning { Color::BLACK } else { TEXT_PRIMARY },
            border: Border {
                color: if is_tuning {
                    TUNE_AMBER
                } else {
                    PANEL_BORDER
                },
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}

pub fn action_button(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Hovered => lighten(ACCENT_BLUE, 0.1),
        button::Status::Pressed => darken(ACCENT_BLUE, 0.1),
        _ => ACCENT_BLUE,
    };

    button::Style {
        background: Some(Background::Color(bg)),
        text_color: Color::WHITE,
        border: Border {
            color: lighten(ACCENT_BLUE, 0.2),
            width: 1.0,
            radius: 3.0.into(),
        },
        shadow: Shadow::default(),
    }
}

// --- Slider style ---

pub fn radio_slider(_theme: &Theme, _status: slider::Status) -> slider::Style {
    slider::Style {
        rail: slider::Rail {
            backgrounds: (
                Background::Color(ACCENT_BLUE),
                Background::Color(SLIDER_EMPTY),
            ),
            width: 6.0,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 3.0.into(),
            },
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: Background::Color(ACCENT_BLUE),
            border_width: 1.0,
            border_color: lighten(ACCENT_BLUE, 0.3),
        },
    }
}

// --- Text input style ---

pub fn freq_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused => ACCENT_BLUE,
        text_input::Status::Hovered => lighten(PANEL_BORDER, 0.1),
        _ => PANEL_BORDER,
    };

    text_input::Style {
        background: Background::Color(PANEL_BG),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        icon: TEXT_DIM,
        placeholder: TEXT_DIM,
        value: FREQ_GREEN,
        selection: ACCENT_BLUE,
    }
}

// --- Color helpers ---

fn lighten(color: Color, amount: f32) -> Color {
    Color::from_rgb(
        (color.r + amount).min(1.0),
        (color.g + amount).min(1.0),
        (color.b + amount).min(1.0),
    )
}

fn darken(color: Color, amount: f32) -> Color {
    Color::from_rgb(
        (color.r - amount).max(0.0),
        (color.g - amount).max(0.0),
        (color.b - amount).max(0.0),
    )
}
