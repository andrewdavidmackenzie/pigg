use iced::widget::{Button, Text};
use iced::{Color, Element};

use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::HardwareView;
use crate::{Message, ToastMessage};

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view(hardware_view: &HardwareView) -> Element<Message> {
    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };
    Button::new(Text::new(hardware_view.hw_model()))
        .on_press(Message::Toast(ToastMessage::HardwareDetailsToast))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(about_button_style.get_button_style())
        .into()
}
