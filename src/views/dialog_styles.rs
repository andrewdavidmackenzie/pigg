use iced::border::Radius;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use iced::widget::text;
use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow};

#[cfg(any(feature = "iroh", feature = "tcp"))]
const RADIUS_10: Radius = Radius {
    top_left: 10.0,
    top_right: 10.0,
    bottom_right: 10.0,
    bottom_left: 10.0,
};

const RADIUS_2: Radius = Radius {
    top_left: 2.0,
    top_right: 2.0,
    bottom_right: 2.0,
    bottom_left: 2.0,
};

pub(crate) const NO_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: iced::Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

pub(crate) const WHITE_BORDER: Border = Border {
    color: Color::WHITE,
    width: 2.0,
    radius: RADIUS_2,
};

pub(crate) const NO_BORDER: Border = Border {
    color: Color::TRANSPARENT,
    width: 0.0,
    radius: RADIUS_2,
};

pub(crate) const MODAL_CANCEL_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.8, 0.0, 0.0, 1.0))),
    text_color: Color::WHITE,
    border: NO_BORDER,
    shadow: NO_SHADOW,
};

pub(crate) const MODAL_CANCEL_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.9, 0.2, 0.2, 1.0))),
    text_color: Color::WHITE,
    border: WHITE_BORDER,
    shadow: NO_SHADOW,
};

pub(crate) const MODAL_CONNECT_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.0, 1.0, 1.0, 1.0))),
    text_color: Color::BLACK,
    border: NO_BORDER,
    shadow: NO_SHADOW,
};

pub(crate) const MODAL_CONNECT_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.0, 0.8, 0.8, 1.0))),
    text_color: Color::BLACK,
    border: WHITE_BORDER,
    shadow: NO_SHADOW,
};

pub(crate) const MODAL_CONTAINER_STYLE: container::Style = container::Style {
    text_color: Some(Color::WHITE),
    background: Some(Background::Color(Color::BLACK)),
    border: WHITE_BORDER,
    shadow: NO_SHADOW,
};

pub(crate) const HYPERLINK_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: Color::from_rgba(0.0, 0.3, 0.8, 1.0),
    border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: RADIUS_2,
    },
    shadow: NO_SHADOW,
};

pub(crate) const HYPERLINK_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: Color::from_rgba(0.1, 0.5, 0.75, 1.0),
    border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: RADIUS_2,
    },
    shadow: NO_SHADOW,
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const INFO_TEXT_STYLE: text::Style = text::Style {
    color: Some(Color::from_rgba(0.8, 0.8, 0.8, 1.0)),
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const TEXT_BOX_CONTAINER_STYLE: container::Style = container::Style {
    text_color: Some(Color::BLACK),
    background: Some(Background::Color(Color::BLACK)),
    border: Border {
        color: Color::from_rgba(0.8, 0.8, 0.8, 1.0),
        width: 2.0,
        radius: RADIUS_10,
    },
    shadow: NO_SHADOW,
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const CONNECTION_ERROR_DISPLAY: text::Style = text::Style {
    color: Some(Color::from_rgba(0.8, 0.0, 0.0, 1.0)),
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const ACTIVE_TAB_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::BLACK)),
    text_color: Color::WHITE,
    border: NO_BORDER,
    shadow: NO_SHADOW,
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const INACTIVE_TAB_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    // Gray text color to show it's inactive
    text_color: Color::from_rgba(0.7, 0.7, 0.7, 1.0),
    border: NO_BORDER,
    shadow: NO_SHADOW,
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const INACTIVE_TAB_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: Color::WHITE,
    border: WHITE_BORDER,
    shadow: NO_SHADOW,
};

#[cfg(any(feature = "iroh", feature = "tcp"))]
pub(crate) const TAB_BAR_STYLE: container::Style = container::Style {
    text_color: Some(Color::BLACK),
    background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
    border: NO_BORDER,
    shadow: NO_SHADOW,
};
