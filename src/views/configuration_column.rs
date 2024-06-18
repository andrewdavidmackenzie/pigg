use crate::custom_widgets::button_style::ButtonStyle;
use crate::{Message, Piggui};
use iced::widget::{Button, Column, Text};
use iced::{Alignment, Color, Element, Length};

/// Construct the view that represents the configuration column
pub fn view(app: &Piggui) -> Element<'static, Message> {
    let file_button_style = ButtonStyle {
        bg_color: Color::new(0.0, 1.0, 1.0, 1.0),
        text_color: Color::BLACK,
        hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0),
        hovered_text_color: Color::WHITE,
        border_radius: 2.0,
    };

    let save_button = Button::new(Text::new("Save Configuration"))
        .style(file_button_style.get_button_style())
        .on_press(Message::Save);
    let load_button = Button::new(Text::new("Load Configuration"))
        .style(file_button_style.get_button_style())
        .on_press(Message::Load);

    let mut configuration_column = Column::new()
        .align_items(Alignment::Start)
        .spacing(10)
        .width(Length::Shrink);
    configuration_column = configuration_column.push(app.layout_selector.view());
    configuration_column = configuration_column.push(save_button);
    configuration_column = configuration_column.push(load_button);

    configuration_column.into()
}
