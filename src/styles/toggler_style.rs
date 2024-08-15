use iced::{widget::toggler, Color, Theme};
use iced::widget::toggler::{Status, Style};

pub struct TogglerStyle {
    pub background: Color,
    pub background_border_width: f32,
    pub background_border_color: Color,
    pub foreground: Color,
    pub foreground_border_width: f32,
    pub foreground_border_color: Color,
    pub active_background: Color,
    pub active_foreground: Color,
    pub active_background_border: Color,
    pub active_foreground_border: Color,
}

impl toggler::Catalog for TogglerStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        todo!()
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        match status {
            Status::Active => {
                Style {
                    background: self.active_background,
                    background_border_width: self.background_border_width,
                    background_border_color: self.active_background_border,
                    foreground: self.active_foreground,
                    foreground_border_width: self.foreground_border_width,
                    foreground_border_color: self.active_foreground_border,
                }
            }

            Status::Hovered =>{
                Style {
                    background: self.active_background,
                    background_border_width: self.background_border_width,
                    background_border_color: self.active_background_border,
                    foreground: self.active_foreground,
                    foreground_border_width: self.foreground_border_width,
                    foreground_border_color: self.active_foreground_border,
                }
            }
        }
    }
}

// impl TogglerStyle {
//     pub fn get_toggler_style(&self) -> iced::widget::toggler::Style {
//         iced::widget::toggler::Toggler::Custom(Box::new(TogglerStyle {
//             background: self.background,
//             background_border_width: 1.0,
//             background_border_color: self.background_border_color,
//             foreground: self.foreground,
//             foreground_border_width: 1.0,
//             foreground_border_color: self.foreground_border_color,
//             active_background: self.active_background,
//             active_foreground: self.active_foreground,
//             active_background_border: self.active_background_border,
//             active_foreground_border: self.active_foreground_border,
//         }))
//     }
// }
