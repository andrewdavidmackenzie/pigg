use crate::custom_widgets::button_style::ButtonStyle;
use crate::views::hardware::hardware_button;
use crate::views::status::status_message;
use crate::views::version::version_button;
use crate::{Gpio, Message};
use iced::widget::{Button, Row};
use iced::{Color, Element, Length};

fn unsaved_status(app: &Gpio) -> Element<Message> {
    let button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border_radius: 4.0,
    };

    match app.unsaved_changes {
        true => Button::new("Unsaved changes").on_press(Message::Save),
        false => Button::new(""),
    }
    .width(Length::Fixed(140.0))
    .style(button_style.get_button_style())
    .into()
}

pub fn info_row(app: &Gpio) -> Element<Message> {
    Row::new()
        .push(version_button(app))
        .push(hardware_button(app))
        .push(unsaved_status(app))
        .push(status_message(app))
        .spacing(20.0)
        .into()
}
