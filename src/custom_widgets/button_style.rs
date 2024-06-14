use iced::{Color, Theme};
use iced::widget::button;

pub struct ButtonStyle {
    pub bg_color: Color,
    pub text_color: Color,
    pub border_radius: f32,
    pub hovered_bg_color: Color,
    pub hovered_text_color: Color,
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.bg_color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: self.border_radius.into(),
            },
            text_color: self.text_color,
            ..Default::default()
        }
    }

    fn hovered(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.hovered_bg_color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: self.border_radius.into(),
            },
            text_color: self.hovered_text_color,
            ..Default::default()
        }
    }
}

impl ButtonStyle {
    pub fn get_button_style(&self) -> iced::widget::theme::Button {
        iced::widget::theme::Button::Custom(Box::new(ButtonStyle {
            bg_color: self.bg_color,
            text_color: self.text_color,
            border_radius: self.border_radius,
            hovered_bg_color: self.hovered_bg_color,
            hovered_text_color: self.hovered_text_color,
        }))
    }
}
