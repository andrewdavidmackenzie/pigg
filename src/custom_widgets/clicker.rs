use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::{Color, Element, Length, Rectangle, Size};

pub struct Clicker {
    radius: f32,
}

impl Clicker {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

pub fn clicker(radius: f32) -> Clicker {
    Clicker::new(radius)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Clicker
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
        layout::Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
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
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: iced::border::Border {
                    radius: self.radius.into(),
                    color: Color::from_rgb8(0, 255, 0),
                    width: 3.0,
                },
                ..renderer::Quad::default()
            },
            Color::WHITE,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Clicker> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(clicker: Clicker) -> Self {
        Self::new(clicker)
    }
}
