use crate::views::layout_selector::LayoutSelector;
use crate::Message;
use iced::widget::{Button, Column, Text};
use iced::{Alignment, Background, Border, Color, Element, Length};
use iced::border::Radius;
use iced::widget::button::Style;

/// Construct the view that represents the configuration column
pub fn view(layout_selector: &LayoutSelector) -> Element<'static, Message> {
    let file_button_style = Style {
        background: Some(Background::Color(Color::new(0.0, 1.0, 1.0, 1.0))),
        text_color: Color::BLACK,
        border: Border {
            color: Default::default(),
            width: 0.0,
            radius: Radius::from(2),
        },
        shadow: Default::default(),
    };
    

    let save_button = Button::new(Text::new("Save Configuration"))
        .style(move |theme, status| file_button_style)
        .on_press(Message::Save);
    let load_button = Button::new(Text::new("Load Configuration"))
        .style(move |theme, status| file_button_style)
        .on_press(Message::Load);

    let mut configuration_column = Column::new()
        .align_x(Alignment::Start)
        .spacing(10)
        .width(Length::Shrink);
    configuration_column = configuration_column.push(layout_selector.view());
    configuration_column = configuration_column.push(save_button);
    configuration_column = configuration_column.push(load_button);

    configuration_column.into()
}
