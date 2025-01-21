use crate::views::menu_bar_button;
use crate::{InfoDialogMessage, Message};
use iced::widget::button;
use iced::{Renderer, Theme};
use iced_aw::menu::Item;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[must_use]
pub fn about() -> String {
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

pub fn about_button<'a>() -> Item<'a, Message, Theme, Renderer> {
    Item::new(
        button("about")
            .on_press(Message::Modal(InfoDialogMessage::AboutDialog))
            .clip(true)
            .height(iced::Length::Shrink)
            .style(menu_bar_button),
    )
}
