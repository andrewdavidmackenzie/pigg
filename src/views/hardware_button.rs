use iced::widget::{Button, Text};
use iced::{Color, Element, Length};

use crate::styles::button_style::ButtonStyle;
use crate::{Message, ToastMessage};

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view() -> Element<'static, Message> {
    let hardware_button = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };

    Button::new(Text::new("Show Hardware Details"))
        .on_press(Message::Toast(ToastMessage::HardwareDetailsToast))
        .width(Length::Fill)
        .style(hardware_button.get_button_style())
        .into()
}
