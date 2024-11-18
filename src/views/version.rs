use crate::views::info_row::{MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE};
use crate::{InfoDialogMessage, Message};
use iced::widget::button::Status::Hovered;
use iced::widget::{Button, Text};
use iced::Element;

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

    Button::new(version_text)
        .on_press(Message::Modal(InfoDialogMessage::VersionModal))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(move |_theme, status| {
            if status == Hovered {
                MENU_BAR_BUTTON_HOVER_STYLE
            } else {
                MENU_BAR_BUTTON_STYLE
            }
        })
        .into()
}
