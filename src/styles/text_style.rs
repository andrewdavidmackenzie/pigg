use iced::widget::text;
use iced::widget::text::{Style};
use iced::{Color, Theme};

pub struct TextStyle {
    pub text_color: Color,
}

impl text::Catalog for TextStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        todo!()
    }

    fn style(&self, item: &Self::Class<'_>) -> Style {
        Style {
            color: Some(self.text_color),
        }
    }
}

impl TextStyle {
    pub fn get_text_color(&self) -> iced::widget::text::Style {
        iced::widget::text::Text::Custom(self.text_color)
    }
}
