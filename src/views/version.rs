use crate::styles::button_style::ButtonStyle;
use crate::{Message, Piggui, ToastMessage};
use iced::widget::{Button, Text};
use iced::{Color, Element};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = "piggui";
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[must_use]
pub fn version() -> String {
    format!(
        "{name} {version}\n\
        Copyright (C) 2024 The {name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {name} Contributors.\n\
        Full source available at: {repository}",
        name = NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
        repository = REPOSITORY,
    )
}

// TODO maybe avoid using Gpio, and pass parameters, or a closure?
pub fn version_button(app: &Piggui) -> Element<Message> {
    let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());
    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };
    Button::new(version_text)
        .on_press(if !app.toast_handler.showing_toast {
            // Add a new toast if `show_toast` is false
            Message::Toast(ToastMessage::VersionToast)
        } else {
            // Close the existing toast if `show_toast` is true
            let index = app.toast_handler.toasts.len() - 1;
            Message::Toast(ToastMessage::Close(index))
        })
        .clip(true)
        .height(iced::Length::Shrink)
        .style(about_button_style.get_button_style())
        .into()
}
