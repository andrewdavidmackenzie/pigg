use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::{Color, Element, Length, Rectangle, Size};

use crate::hw_definition::PinLevel;

pub struct Led {
    height: f32,
    level: Option<PinLevel>,
}

impl Led {
    pub fn new(height: f32, _width: f32, level: Option<PinLevel>) -> Self {
        Self { height, level }
    }
}

pub fn led(height: f32, width: f32, level: Option<PinLevel>) -> Led {
    Led::new(height, width, level)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Led
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.height * 2.0, self.height * 2.0))
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let color = match self.level {
            None => Color::BLACK,
            Some(false) => Color::new(0.0, 0.3, 0.0, 1.0),
            Some(true) => Color::new(0.0, 0.7, 0.0, 1.0),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: iced::border::Border {
                    radius: self.height.into(),
                    ..Default::default()
                },
                ..renderer::Quad::default()
            },
            color,
        );
    }
}

impl<Message, Theme, Renderer> From<Led> for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(circle: Led) -> Self {
        Self::new(circle)
    }
}
