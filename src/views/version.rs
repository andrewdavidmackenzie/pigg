use crate::{Message, ModalMessage};
use iced::widget::{Button, Text};
use iced::{Background, Border, Color, Element};
use iced::border::Radius;
use iced::widget::button::Style;


const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[must_use]
pub fn version() -> String {
    format!(
        "{bin_name} {version}\n\
        Copyright (C) 2024 The {pkg_name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {pkg_name} Contributors.\n\
        Full source available at: {repository}",
        bin_name = BIN_NAME,
        pkg_name = PKG_NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
        repository = REPOSITORY,
    )
}

pub fn version_button() -> Element<'static, Message> {
    let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());
    let about_button_style = Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border:  Border {
            radius: Radius::from(4),
            color: Default::default(),
            width: 0.0,
        },
        shadow: Default::default(),
    };
    Button::new(version_text)
        .on_press(Message::ModalHandle(ModalMessage::VersionModal))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(move |_theme, _status| about_button_style)
        .into()
}
