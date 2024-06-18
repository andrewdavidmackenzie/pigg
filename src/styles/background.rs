use iced::widget::container;

use crate::Message;
use iced::theme::Container;
use iced::{Background, Color};

#[derive(Default)]
pub struct BackgroundColor {
    color: Color,
}

impl BackgroundColor {
    const fn new(color: Color) -> Self {
        Self { color }
    }
}

impl container::StyleSheet for BackgroundColor {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::from(self.color)),
            ..Default::default()
        }
    }
}

pub trait SetAppearance {
    fn set_background(self, color: Color) -> Self;
}

impl SetAppearance for container::Container<'_, Message> {
    fn set_background(self, color: Color) -> Self {
        self.style(Container::Custom(Box::new(BackgroundColor::new(color))))
    }
}
