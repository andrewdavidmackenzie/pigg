use crate::hw_definition::PinLevel;
use crate::widgets::led::Status::{Active, Disabled, Hovered};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::{advanced::Clipboard, advanced::Shell, touch, Theme};
use iced::{event, mouse, Event};
use iced::{Color, Element, Length, Rectangle, Size};

pub struct Led<'a, Message, Theme = crate::Theme>
where
    Theme: Catalog,
{
    radius: f32,
    on_press: Option<Message>,
    on_release: Option<Message>,
    level: Option<PinLevel>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme> Led<'a, Message, Theme>
where
    Theme: Catalog,
{
    pub fn new(height: f32, level: Option<PinLevel>) -> Self {
        Self {
            radius: height,
            on_press: None,
            on_release: None,
            level,
            class: Theme::default(),
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

    /// Sets the style of the [`Led`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }
}

pub fn led<'a, Message>(height: f32, level: Option<PinLevel>) -> Led<'a, Message> {
    Led::new(height, level)
}

#[allow(missing_debug_implementations)]
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Led<'_, Message, Theme>
where
    Message: Clone,
    Theme: Catalog,
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
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let status = if self.on_press.is_none() {
            Disabled
        } else if cursor.is_over(layout.bounds()) {
            Hovered
        } else {
            Active
        };

        let style = theme.style(&self.class, status);

        let color = match self.level {
            None => Color::BLACK,
            Some(false) => style.off_color,
            Some(true) => style.on_color,
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: iced::border::Border {
                    radius: self.radius.into(),
                    width: style.border_hover_width,
                    color: if status == Hovered {
                        style.border_hover_color
                    } else {
                        Color::TRANSPARENT
                    },
                },
                ..renderer::Quad::default()
            },
            color,
        );
    }

    fn on_event(
        &mut self,
        _tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(on_press) = self.on_press.clone() {
                    if cursor.is_over(layout.bounds()) {
                        shell.publish(on_press);
                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_release) = self.on_release.clone() {
                    if cursor.is_over(layout.bounds()) {
                        shell.publish(on_release);
                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => {}
            _ => {}
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Led<'a, Message, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: renderer::Renderer,
{
    fn from(led: Led<'a, Message, Theme>) -> Self {
        Self::new(led)
    }
}

/// The possible status of a [`Led`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Led`] can be interacted with.
    Active,
    /// The [`Led`] is being hovered.
    Hovered,
    /// The [`Led`] is disabled.
    Disabled,
}

/// The appearance of an [`Led`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub off_color: Color,
    pub on_color: Color,
    pub border_hover_color: Color,
    pub border_hover_width: f32,
}

/// The theme catalog of a [`Led`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Led`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Led`].
pub fn default(_theme: &Theme, _status: Status) -> Style {
    Style {
        off_color: Color::from_rgba(0.0, 0.3, 0.0, 1.0),
        on_color: Color::from_rgba(0.0, 0.7, 0.0, 1.0),
        border_hover_color: Color::WHITE,
        border_hover_width: 3.0,
    }
}
