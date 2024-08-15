use iced::widget::container;

use iced::widget::container::Style;
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

impl container::Catalog for BackgroundColor {
    type Class<'a> = iced::Theme;
    fn default<'a>() -> Self::Class<'a> {
        todo!()
    }

    fn style(&self, _class: &Self::Class<'_>) -> Style {
        Style {
            background: Some(Background::from(self.color)),
            ..Default::default()
        }
    }
}

// pub trait SetAppearance {
//     fn set_background(self, color: Color) -> Self;
// }

// impl SetAppearance for container::Container<'_, Message> {
//     fn set_background(self, color: Color) -> Self {
//         self.style(container::Container::Custom(Box::new(
//             BackgroundColor::new(color),
//         )))
//     }
// }
