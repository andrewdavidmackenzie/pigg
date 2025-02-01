use crate::views::dialog_styles::NO_SHADOW;
use crate::views::hardware_view::PIN_BUTTON_RADIUS;
use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::toggler::Status::Hovered;
use iced::widget::{button, container, pick_list, toggler};
use iced::{Background, Border, Color, Shadow, Theme};
use iced_aw::style::colors::WHITE;

const PIN_RADIUS: Radius = Radius {
    top_left: PIN_BUTTON_RADIUS,
    top_right: PIN_BUTTON_RADIUS,
    bottom_right: PIN_BUTTON_RADIUS,
    bottom_left: PIN_BUTTON_RADIUS,
};

const PIN_BORDER: Border = Border {
    color: Color::WHITE,
    width: 1.0,
    radius: PIN_RADIUS,
};

const PIN_NO_BORDER: Border = Border {
    color: Color::TRANSPARENT,
    width: 1.0,
    radius: PIN_RADIUS,
};

const PIN_BORDER_HOVER: Border = Border {
    color: Color::WHITE,
    width: 3.0,
    radius: PIN_RADIUS,
};

pub fn toggler_style(_theme: &Theme, status: toggler::Status) -> toggler::Style {
    toggler::Style {
        background: Color::from_rgba(0.0, 0.3, 0.0, 1.0), // Dark green background (inactive)
        background_border_width: 3.0,
        background_border_color: match status {
            Hovered { .. } => WHITE,
            _ => Color::from_rgba(0.0, 0.2, 0.0, 1.0),
        },
        foreground: Color::from_rgba(1.0, 0.9, 0.8, 1.0), // Light yellowish foreground
        foreground_border_width: 3.0,
        foreground_border_color: Color::from_rgba(0.9, 0.9, 0.9, 1.0), // Light gray foreground border
    }
}

pub fn picklist_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        border: Border {
            color: if status == pick_list::Status::Hovered {
                Color::WHITE
            } else {
                Color::TRANSPARENT
            },
            width: 3.0,
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
        },
        ..pick_list::default(theme, status)
    }
}

pub(crate) const TOOLTIP_STYLE: container::Style = container::Style {
    text_color: Some(Color::WHITE),
    background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 1.0))),
    border: Border {
        color: Color::from_rgba(0.7, 0.7, 0.7, 1.0),
        width: 1.0,
        radius: Radius {
            top_left: 4.0,
            top_right: 4.0,
            bottom_right: 4.0,
            bottom_left: 4.0,
        },
    },
    shadow: NO_SHADOW,
};

/// Return a style used to draw a pin, based on it's name and if it has options or not and
/// the Hover status.
pub(crate) fn get_pin_style(status: Status, pin_name: &str, options: bool) -> button::Style {
    let (pin_color, text_color) = match pin_name {
        "3V3" => (Color::from_rgba(1.0, 0.92, 0.016, 1.0), Color::BLACK), // Yellow
        "5V" => (Color::from_rgba(1.0, 0.0, 0.0, 1.0), Color::BLACK),     // Red,
        "Ground" => (Color::BLACK, Color::WHITE),
        "GPIO2" | "GPIO3" => (Color::from_rgba(0.678, 0.847, 0.902, 1.0), Color::BLACK), // Light Blue
        "GPIO7" | "GPIO8" | "GPIO9" | "GPIO10" | "GPIO11" => {
            (Color::from_rgba(0.933, 0.510, 0.933, 1.0), Color::WHITE)
        } // Pink
        "GPIO14" | "GPIO15" => (Color::from_rgba(0.0, 0.502, 0.0, 1.0), Color::WHITE),   // Green
        _ => (Color::from_rgba(1.0, 0.647, 0.0, 1.0), Color::WHITE),                     // Orange
    };

    let border = if !options {
        PIN_NO_BORDER
    } else if status == Status::Hovered {
        PIN_BORDER_HOVER
    } else {
        PIN_BORDER
    };

    button::Style {
        background: Some(Background::Color(pin_color)),
        text_color,
        border,
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset: iced::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
    }
}
