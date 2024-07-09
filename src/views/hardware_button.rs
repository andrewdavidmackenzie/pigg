use iced::widget::{Button, Text};
use iced::{Color, Element, Length};

use crate::styles::button_style::ButtonStyle;
use crate::{Message, ToastMessage};

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view() -> Element<'static, Message> {
    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };
    Button::new(Text::new("Show Hardware Details"))
        .on_press(Message::Toast(ToastMessage::HardwareDetailsToast))
        .width(Length::Fill)
        .style(about_button_style.get_button_style())
        .into()
}
