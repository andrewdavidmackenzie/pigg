use crate::custom_widgets::button_style::ButtonStyle;
use crate::styles::background::SetAppearance;
use crate::views::hardware_button;
use crate::views::version::version_button;
use crate::{Message, Piggui};
use iced::widget::{container, Button, Row};
use iced::{Color, Element, Length};

fn unsaved_status(app: &Piggui) -> Element<Message> {
    let button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::new(1.0, 0.0, 0.0, 1.0),
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border_radius: 4.0,
    };

    match app.unsaved_changes {
        true => Button::new("Unsaved changes").on_press(Message::Save),
        false => Button::new(""),
    }
    .width(Length::Fixed(160.0))
    .style(button_style.get_button_style())
    .into()
}

/// Create the view that represents the info row at the bottom of the window
pub fn view(app: &Piggui) -> Element<Message> {
    container(
        Row::new()
            .push(version_button(app))
            .push(hardware_button::view(app))
            .push(unsaved_status(app))
            .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
            .push(app.status_row.view().map(Message::StatusRow))
            .spacing(20.0)
            .padding([0.0, 0.0, 0.0, 0.0]),
    )
    .set_background(Color::from_rgb8(45, 45, 45))
    .into()
}
