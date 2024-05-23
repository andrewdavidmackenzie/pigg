use iced::widget::button;
use iced::{Color, Theme};

pub struct CustomButton {
    pub bg_color: Color,
    pub text_color: Color,
}

impl button::StyleSheet for CustomButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.bg_color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 50.0.into(),
            },
            text_color: self.text_color,
            ..Default::default()
        }
    }
}

impl CustomButton {
    pub fn get_button_style(&self) -> iced::widget::theme::Button {
        iced::widget::theme::Button::Custom(Box::new(CustomButton {
            bg_color: self.bg_color,
            text_color: self.text_color,
        }))
    }
}
