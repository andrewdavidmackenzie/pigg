
use iced::{Border, Color, Theme};
use iced::widget::container;

pub struct ContainerStyle {
    pub border_color: Color,
}

impl container::StyleSheet for ContainerStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border: Border {
                color: self.border_color,
                width: 2.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        }
    }
}

impl ContainerStyle {
    pub fn get_container_style(&self) -> iced::widget::theme::Container {
        iced::widget::theme::Container::Custom(Box::new(ContainerStyle {
            border_color: self.border_color,
        }))
    }
}