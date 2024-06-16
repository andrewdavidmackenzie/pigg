use crate::views::hardware::hardware_button;
use crate::views::version::version_button;
use crate::{Gpio, Message};
use iced::widget::Row;
use iced::Element;

pub fn info_row(app: &Gpio) -> Element<Message> {
    Row::new()
        .push(version_button(app))
        .push(hardware_button(app))
        .spacing(20.0)
        .into()
}
