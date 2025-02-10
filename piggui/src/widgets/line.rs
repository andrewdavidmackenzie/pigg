use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::Element;
use iced::{Color, Length, Rectangle, Size};

pub struct Line {
    length: f32,
}

impl Line {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

pub fn line(length: f32) -> Line {
    Line::new(length)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Line
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Fixed(1f32),
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.length, 1.0))
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
                ..renderer::Quad::default()
            },
            Color::WHITE,
        );
    }
}

impl<Message, Theme, Renderer> From<Line> for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(line: Line) -> Self {
        Self::new(line)
    }
}
