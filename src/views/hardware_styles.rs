use crate::views::dialog_styles::NO_SHADOW;
use crate::views::pin_state::CHART_WIDTH;
use iced::border::Radius;
use iced::widget::button::Status;
use iced::widget::{button, container, toggler};
use iced::{Background, Border, Color, Shadow};
use iced_aw::style::colors::WHITE;
// WIDTHS
pub(crate) const PIN_BUTTON_DIAMETER: f32 = 28.0;
const PIN_BUTTON_RADIUS: f32 = PIN_BUTTON_DIAMETER / 2.0;
pub(crate) const PIN_ARROW_LINE_WIDTH: f32 = 20.0;
pub(crate) const PIN_ARROW_CIRCLE_RADIUS: f32 = 5.0;
pub(crate) const PIN_ROW_WIDTH: f32 =
    PIN_ARROW_LINE_WIDTH + (PIN_ARROW_CIRCLE_RADIUS * 2.0) + PIN_BUTTON_DIAMETER;
pub(crate) const PIN_NAME_WIDTH: f32 = 80.0; // for some longer pin names
pub(crate) const TOGGLER_SIZE: f32 = 28.0;
pub(crate) const TOGGLER_WIDTH: f32 = 95.0; // Just used to calculate Pullup width
pub(crate) const CLICKER_WIDTH: f32 = 13.0;
// We want the pullup on an Input to be the same width as the clicker + toggler on an Output
pub(crate) const PULLUP_WIDTH: f32 = TOGGLER_WIDTH + CLICKER_WIDTH - 3.0;
pub(crate) const LED_WIDTH: f32 = 14.0;
pub(crate) const WIDGET_ROW_SPACING: f32 = 5.0;
pub(crate) const PIN_WIDGET_ROW_WIDTH: f32 =
    PULLUP_WIDTH + WIDGET_ROW_SPACING + LED_WIDTH + WIDGET_ROW_SPACING + CHART_WIDTH;

pub(crate) const SPACE_BETWEEN_PIN_COLUMNS: f32 = 10.0;

// Export these two, so they can be used to calculate overall window size
// pub const BCM_PIN_LAYOUT_WIDTH: f32 = PIN_VIEW_SIDE_WIDTH; // One pin row per row

// Board Layout has two pin rows per row, with spacing between them
// pub const BOARD_PIN_LAYOUT_WIDTH: f32 =
//     PIN_VIEW_SIDE_WIDTH + PIN_VIEW_SIDE_WIDTH + BOARD_LAYOUT_WIDTH_BETWEEN_PIN_ROWS;

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

const PIN_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: iced::Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

const DARK_GREEN: Color = Color::from_rgba(0.0, 0.3, 0.0, 1.0);

pub(crate) const TOGGLER_STYLE: toggler::Style = toggler::Style {
    background: DARK_GREEN, // Dark green background (inactive)
    background_border_width: 2.0,
    background_border_color: Color::from_rgba(0.0, 0.2, 0.0, 1.0), // Darker green border
    foreground: Color::from_rgba(1.0, 0.9, 0.8, 1.0),              // Light yellowish foreground
    foreground_border_width: 1.0,
    foreground_border_color: Color::from_rgba(0.9, 0.9, 0.9, 1.0), // Light gray foreground border
};

pub(crate) const TOGGLER_HOVER_STYLE: toggler::Style = toggler::Style {
    background: DARK_GREEN, // Dark green background (inactive)
    background_border_width: 2.0,
    background_border_color: WHITE,
    foreground: Color::from_rgba(1.0, 0.9, 0.8, 1.0), // Light yellowish foreground (inactive)
    foreground_border_width: 1.0,
    foreground_border_color: Color::from_rgba(0.9, 0.9, 0.9, 1.0), // Light gray foreground border (inactive)
};

const RADIUS_4: Radius = Radius {
    top_left: 4.0,
    top_right: 4.0,
    bottom_right: 4.0,
    bottom_left: 4.0,
};

const TOOLTIP_BORDER: Border = Border {
    color: Color::from_rgba(0.7, 0.7, 0.7, 1.0),
    width: 1.0,
    radius: RADIUS_4,
};

pub(crate) const TOOLTIP_STYLE: container::Style = container::Style {
    text_color: Some(Color::WHITE),
    background: Some(Background::Color(Color::from_rgba(0.3, 0.3, 0.3, 1.0))),
    border: TOOLTIP_BORDER,
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
        shadow: PIN_SHADOW,
    }
}
