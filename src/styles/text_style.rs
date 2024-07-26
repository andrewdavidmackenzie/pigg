use iced::{Color, Theme};
use iced::widget::text;
use iced::widget::text::Appearance;
use crate::styles::button_style::ButtonStyle;

pub struct TextStyle {
    pub text_color: Color,
}

impl text::StyleSheet for TextStyle {
    type Style = Theme;

    fn appearance(&self, _style: Self::Style) -> Appearance {
        Appearance {
            color: Some(self.text_color)
        }
    }
}

impl TextStyle {
    pub fn get_text_color(&self) -> iced::widget::theme::Text {
        iced::widget::theme::Text::Color(self.text_color)
    }
}