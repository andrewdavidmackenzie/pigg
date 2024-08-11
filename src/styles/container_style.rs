use iced::widget::container;
use iced::{Background, Border, Color, Theme};
use iced::widget::container::Style;

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

    fn style(&self, class: &Self::Class<'_>) -> Style {
        container::Style {
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

impl ContainerStyle {
    pub fn get_container_style(&self) -> iced::widget::container::Style {
        iced::widget::container::Container::Custom(Box::new(ContainerStyle {
            background_color: self.background_color,
            border_color: self.border_color,
            border_width: self.border_width,
            border_radius: self.border_radius,
        }))
    }
}
