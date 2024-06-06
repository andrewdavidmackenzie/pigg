use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced::mouse;
use iced::{Color, Element, Length, Rectangle, Size};
use plotters_iced::{Chart, ChartBuilder, DrawingBackend};

use crate::{Message, PinState};

// TODO see if we can do with references to avoid duplicating all the pin's history
pub struct Waveform {
    height: f32,
    width: f32,
    pin_state: PinState,
}

impl Waveform {
    pub fn new(height: f32, width: f32, pin_state: &PinState) -> Self {
        Self {
            height,
            width,
            pin_state: pin_state.clone(),
        }
    }
}

impl Chart<Message> for Waveform {
    type State = ();
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {
        //build your chart here, please refer to plotters for more details
    }
}

#[allow(dead_code)]
pub fn waveform(height: f32, width: f32, pin_state: &PinState) -> Waveform {
    Waveform::new(height, width, pin_state)
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Waveform
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
        layout::Node::new(Size::new(self.width, self.height * 2.0))
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
        let color = match self.pin_state.get_level() {
            None => Color::BLACK,
            Some(false) => Color::new(0.0, 0.502, 0.0, 1.0),
            Some(true) => Color::new(1.0, 0.0, 0.0, 1.0),
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

impl<'a, Message, Theme, Renderer> From<Waveform> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(wf: Waveform) -> Self {
        Self::new(wf)
    }
}
