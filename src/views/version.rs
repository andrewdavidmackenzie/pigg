use crate::{Message, ModalMessage};
use iced::widget::{Button, button, Text};
use iced::{Background, Border, Color, Element, Shadow};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[must_use]
pub fn version() -> String {
    format!(
        "{bin_name} {version}\n\
        Copyright (C) 2024 The {pkg_name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {pkg_name} Contributors",
        bin_name = BIN_NAME,
        pkg_name = PKG_NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
    )
}

pub fn version_button() -> Element<'static, Message> {
    let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());
    let about_button_style = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        // bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        // hovered_bg_color: Color::TRANSPARENT,
        // hovered_text_color: Color::WHITE,
        // border_radius: 4.0,
        shadow: Shadow {
            color: Color::TRANSPARENT,
            offset:  iced::Vector { x: 0.0, y: 0.0 },
            blur_radius: 0.0,
        },
    };
    Button::new(version_text)
        .on_press(Message::ModalHandle(ModalMessage::VersionModal))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(move |theme, status | {
            about_button_style
        })
        .into()
}
