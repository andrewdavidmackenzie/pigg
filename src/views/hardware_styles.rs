use crate::hw_definition::description::PinDescription;
use crate::views::dialog_styles::NO_SHADOW;
use crate::views::pin_state::CHART_WIDTH;
use iced::border::Radius;
use iced::widget::{button, container, toggler};
use iced::{Background, Border, Color, Shadow};
use iced_aw::style::colors::WHITE;

// WIDTHS
pub(crate) const PIN_BUTTON_WIDTH: f32 = 28.0;
const PIN_BUTTON_RADIUS: f32 = PIN_BUTTON_WIDTH / 2.0;
pub(crate) const PIN_ARROW_LINE_WIDTH: f32 = 20.0;
pub(crate) const PIN_ARROW_CIRCLE_RADIUS: f32 = 5.0;
pub(crate) const PIN_ARROW_WIDTH: f32 = PIN_ARROW_LINE_WIDTH + PIN_ARROW_CIRCLE_RADIUS * 2.0;
pub(crate) const PIN_NAME_WIDTH: f32 = 60.0;
pub(crate) const PIN_OPTION_WIDTH: f32 = 135.0; // Pi needs 135px
pub(crate) const TOGGLER_SIZE: f32 = 30.0;
const TOGGLER_WIDTH: f32 = 95.0; // Just used to calculate Pullup width
pub(crate) const CLICKER_WIDTH: f32 = 13.0;
// We want the pullup on an Input to be the same width as the clicker + toggler on an Output
pub(crate) const PULLUP_WIDTH: f32 = TOGGLER_WIDTH + CLICKER_WIDTH;
pub(crate) const LED_WIDTH: f32 = 16.0;
pub(crate) const WIDGET_ROW_SPACING: f32 = 5.0;
pub(crate) const PIN_WIDGET_ROW_WIDTH: f32 =
    PULLUP_WIDTH + WIDGET_ROW_SPACING + LED_WIDTH + WIDGET_ROW_SPACING + CHART_WIDTH;

// const PIN_VIEW_SIDE_WIDTH: f32 = PIN_BUTTON_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_ARROW_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_NAME_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_OPTION_WIDTH;

pub(crate) const SPACE_BETWEEN_PIN_COLUMNS: f32 = 10.0;

// Export these two, so they can be used to calculate overall window size
// pub const BCM_PIN_LAYOUT_WIDTH: f32 = PIN_VIEW_SIDE_WIDTH; // One pin row per row

// Board Layout has two pin rows per row, with spacing between them
// pub const BOARD_PIN_LAYOUT_WIDTH: f32 =
//     PIN_VIEW_SIDE_WIDTH + PIN_VIEW_SIDE_WIDTH + BOARD_LAYOUT_WIDTH_BETWEEN_PIN_ROWS;

pub(crate) const SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;

const PIN_RADIUS: Radius = Radius {
    top_left: PIN_BUTTON_RADIUS,
    top_right: PIN_BUTTON_RADIUS,
    bottom_right: PIN_BUTTON_RADIUS,
    bottom_left: PIN_BUTTON_RADIUS,
};

const PIN_BORDER: Border = Border {
    color: Color::TRANSPARENT,
    width: 0.0,
    radius: PIN_RADIUS,
};

const PIN_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: iced::Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

const V3_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(1.0, 0.92, 0.016, 1.0))),
    text_color: Color::BLACK,
    border: PIN_BORDER,
    // hovered_bg_color: Color::new(1.0, 1.0, 0.0, 1.0),
    // hovered_text_color: Color::BLACK,
    shadow: PIN_SHADOW,
};

const V5_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(1.0, 0.0, 0.0, 1.0))),
    text_color: Color::BLACK,
    border: PIN_BORDER,
    // hovered_bg_color: Color::new(1.0, 0.0, 0.0, 1.0),
    // hovered_text_color: Color::BLACK,
    shadow: PIN_SHADOW,
};

const GND_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::BLACK)),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::BLACK,
    shadow: PIN_SHADOW,
};

const GPIO_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(
        0.678, 0.847, 0.902, 1.0,
    ))),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::new(0.678, 0.847, 0.902, 1.0),
    shadow: PIN_SHADOW,
};

const GPIO7_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(
        0.933, 0.510, 0.933, 1.0,
    ))),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::new(0.933, 0.510, 0.933, 1.0),
    shadow: PIN_SHADOW,
};

const GPIO14_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.0, 0.502, 0.0, 1.0))),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::new(0.0, 0.502, 0.0, 1.0),
    shadow: PIN_SHADOW,
};

const ID_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(
        0.502, 0.502, 0.502, 1.0,
    ))),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::new(0.502, 0.502, 0.502, 1.0),
    shadow: PIN_SHADOW,
};

const DEFAULT_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(1.0, 0.647, 0.0, 1.0))),
    text_color: Color::WHITE,
    border: PIN_BORDER,
    // hovered_bg_color: Color::WHITE,
    // hovered_text_color: Color::new(1.0, 0.647, 0.0, 1.0),
    shadow: PIN_SHADOW,
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

pub(crate) fn get_pin_style(pin_description: &PinDescription) -> button::Style {
    match pin_description.name.as_ref() {
        "3V3" => V3_BUTTON_STYLE,
        "5V" => V5_BUTTON_STYLE,
        "Ground" => GND_BUTTON_STYLE,
        "GPIO2" | "GPIO3" => GPIO_BUTTON_STYLE,
        "GPIO7" | "GPIO8" | "GPIO9" | "GPIO10" | "GPIO11" => GPIO7_BUTTON_STYLE,
        "GPIO14" | "GPIO15" => GPIO14_BUTTON_STYLE,
        "ID_SD" | "ID_SC" => ID_BUTTON_STYLE,
        _ => DEFAULT_BUTTON_STYLE,
    }
}
