use iced::widget::button;
use iced::widget::button::{Status, Style};
use iced::{Color, Theme};

pub struct ButtonStyle {
    pub bg_color: Color,
    pub text_color: Color,
    pub border_radius: f32,
    pub hovered_bg_color: Color,
    pub hovered_text_color: Color,
}

impl button::Catalog for ButtonStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        todo!()
    }

    fn style(&self, _class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => button::Style {
                background: Some(iced::Background::Color(self.bg_color)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: self.border_radius.into(),
                },
                text_color: self.text_color,
                ..Default::default()
            },
            Status::Hovered | Status::Pressed | Status::Disabled => button::Style {
                background: Some(iced::Background::Color(self.hovered_bg_color)),
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: self.border_radius.into(),
                },
                text_color: self.hovered_text_color,
                ..Default::default()
            },
        }
    }
}