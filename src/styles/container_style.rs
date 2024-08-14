use iced::widget::container;
use iced::widget::container::Style;
use iced::{Background, Border, Color, Theme};

pub struct ContainerStyle {
    pub border_color: Color,
    pub background_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
}

impl container::Catalog for ContainerStyle {
    type Class<'a> = Theme;
    fn default<'a>() -> Self::Class<'a> {
        todo!()
    }

    fn style(&self, _class: &Self::Class<'_>) -> Style {
        Style {
            background: Some(Background::Color(self.background_color)),
            border: Border {
                color: self.border_color,
                width: self.border_width,
                radius: self.border_radius.into(),
            },
            ..Default::default()
        }
    }
}
