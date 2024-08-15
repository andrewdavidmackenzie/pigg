use crate::Message;
use iced::widget::Button;
use iced::{Border, Color, Element, Length, Background};
use iced::border::Radius;
use iced::widget::button::Style;

/// Create the view that represents the status of unsaved changes in the info row
pub fn view(unsaved_changes: bool) -> Element<'static, Message> {
    let button_style = Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: Color::new(1.0, 0.647, 0.0, 0.7),
        border: Border {
            color: Default::default(),
            width: 0.0,
            radius: Radius::from(4),
        },
        shadow: Default::default(),
    };
    match unsaved_changes {
        true => Button::new("Unsaved changes").on_press(Message::Save),
        false => Button::new(""),
    }
    .width(Length::Fixed(160.0))
    .style(move |_theme, _status| button_style)
    .into()
}
