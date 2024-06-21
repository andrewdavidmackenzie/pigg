use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::{advanced::Clipboard, advanced::Shell, touch};
use iced::{event, mouse, Event};
use iced::{Color, Element, Length, Rectangle, Size};


pub struct Clicker<Message> {
    radius: f32,
    on_press: Option<Message>,
    on_release: Option<Message>,
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct State {
    is_pressed: bool,
    color: Color,
}

impl Default for State {
    fn default() -> Self {
        Self {
            // Default
            is_pressed: false,
            color: Color::WHITE,
        }
    }
}


impl<Message> Clicker<Message> {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            on_press: None,
            on_release: None,
        }
    }
        pub fn on_press(mut self, on_press: Message) -> Self {
            self.on_press = Some(on_press);
            self
        }

        pub fn on_release(mut self, on_release: Message) -> Self {
            self.on_release = Some(on_release);
            self
        }
    }


pub fn clicker<Message, Renderer>(radius: f32) -> Clicker< Message> {
    Clicker::new(radius)
}

#[allow(missing_debug_implementations)]
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Clicker<Message>
    where
        Message: Clone,
        Renderer: renderer::Renderer,
{

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }


    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: iced::border::Border {
                    radius: self.radius.into(),
                    width: 3.0,
                    color: Color::from_rgb8(0, 255, 0),
                },
                ..renderer::Quad::default()
            },
            state.color, // State color
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {

                if let Some(on_press) = self.on_press.clone() {
                        state.is_pressed = true;
                        state.color = Color::BLACK;
                        shell.publish(on_press);
                        return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_release) = self.on_release.clone() {
                    if cursor.is_over(layout.bounds()) {
                        state.is_pressed = false;
                        state.color = Color::WHITE;
                        shell.publish(on_release);
                        return event::Status::Captured;
                    }
                }
            }

            Event::Touch(touch::Event::FingerLost { .. }) => {
                state.is_pressed = false;
                state.color = Color::WHITE;
            }
            _ => {}
        }

        event::Status::Ignored
    }

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
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if cursor.is_over(bounds) {
            if state.is_pressed {
                mouse::Interaction::Grabbing
            } else {
                mouse::Interaction::Pointer
            }
        } else {
            if state.is_pressed {
                mouse::Interaction::Grab
            } else {
                mouse::Interaction::default()
            }
        }
    }
}
impl<'a, Message, Theme, Renderer> From<Clicker<Message>> for Element<'a,  Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Renderer: renderer::Renderer,
{
    fn from(clicker: Clicker<Message>) -> Self {
        Element::new(clicker)
    }
}


