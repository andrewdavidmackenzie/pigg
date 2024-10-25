use crate::widgets::spinner::circular::StyleSheet;
use iced::application::Appearance;
use iced::{Color, Theme};

pub struct TextStyle {
    pub text_color: Color,
}

impl StyleSheet for TextStyle {
    type Style = Theme;

    fn appearance(&self, _style: Self::Style) -> Appearance {
        Appearance {
            color: Some(self.text_color),
        }
    }
}

impl TextStyle {
    pub fn get_text_color(&self) -> iced::widget::theme::Text {
        iced::widget::theme::Text::Color(self.text_color)
    }
}
