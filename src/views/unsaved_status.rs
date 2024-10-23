use crate::styles::button_style::ButtonStyle;
use crate::Message;
use iced::widget::Button;
use iced::{Color, Element, Length};

/// Create the view that represents the status of unsaved changes in the info row
pub fn view(unsaved_changes: bool) -> Element<'static, Message> {
    let unsaved_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(1.0, 0.647, 0.0, 0.7),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(1.0, 0.647, 0.0, 1.0),
        border_radius: 4.0,
    };

    let saved_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(1.0, 0.647, 0.0, 1.0),
        border_radius: 4.0,
    };

    match unsaved_changes {
        true => Button::new("config: unsaved changes")
            .on_press(Message::Save)
            .style(unsaved_style.get_button_style()),
        false => Button::new("config").style(saved_style.get_button_style()),
    }
    .width(Length::Fixed(160.0))
    .into()
}
