use iced::widget::{Button, Text};
use iced::{Color, Element};

use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::HardwareView;
use crate::{Message, ToastMessage};

#[must_use]
pub fn hw_description(hardware_view: &HardwareView) -> String {
    if let Some(hardware_description) = &hardware_view.hardware_description {
        format!(
            "Hardware: {}\nRevision: {}\nSerial: {}\nModel: {}",
            hardware_description.details.hardware,
            hardware_description.details.revision,
            hardware_description.details.serial,
            hardware_description.details.model,
        )
    } else {
        "No Hardware connected".to_string()
    }
}

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view(hardware_view: &HardwareView) -> Element<Message> {
    let hw_text = if let Some(hardware_description) = &hardware_view.hardware_description {
        hardware_description.details.model.clone()
    } else {
        "No Hardware".to_string()
    };

    let about_button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };
    Button::new(Text::new(hw_text))
        .on_press(Message::Toast(ToastMessage::HardwareDetailsToast))
        .clip(true)
        .height(iced::Length::Shrink)
        .style(about_button_style.get_button_style())
        .into()
}
