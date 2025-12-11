use std::fmt;
use std::time::{Duration, Instant};

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::event::Event;
use iced::mouse;
use iced::theme;
use iced::widget::{button, column, container, row, rule, space, text};
use iced::window;
use iced::{Alignment, Element, Fill, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

pub const DEFAULT_TIMEOUT: u64 = u64::MAX / 3;

/// Represents different statuses a toast can have
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    /// Used for general information
    Primary,
    /// Used for less prominent information
    Secondary,
    /// Indicating successful operation
    Success,
    /// Indicating error or warning
    Danger,
}

impl Status {
    pub const ALL: &'static [Self] = &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Primary => "Primary",
            Status::Secondary => "Secondary",
            Status::Success => "Success",
            Status::Danger => "Danger",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Toast {
    pub title: String,
    pub body: String,
    pub status: Status,
}

pub struct Manager<'a, Message> {
    content: Element<'a, Message>,
    toasts: Vec<Element<'a, Message>>,
    timeout_secs: u64,
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
    Message: 'a + Clone,
{
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                container(column![
                    container(
                        row![
                            text(toast.title.as_str()),
                            space::horizontal(),
                            button("X").on_press(on_close(index)).padding(3),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .width(Fill)
                    .padding(5)
                    .style(match toast.status {
                        Status::Primary => primary,
                        Status::Secondary => secondary,
                        Status::Success => success,
                        Status::Danger => danger,
                    }),
                    rule::horizontal(1),
                    container(text(toast.body.as_str())).width(Fill).padding(5),
                ])
                .width(550)
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            timeout_secs: DEFAULT_TIMEOUT,
            on_close: Box::new(on_close),
        }
    }

    pub fn timeout(self, seconds: u64) -> Self {
        Self {
            timeout_secs: seconds,
            ..self
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Manager<'_, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        widget::tree::Tag::of::<Marker>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toasts.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        // Invalidating removed instants to None allows us to remove
        // them here so that diffing for removed / new toast instants
        // is accurate
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                instants.truncate(new);
            }
            (old, new) if old < new => {
                instants.extend(std::iter::repeat(Some(Instant::now())).take(new - old));
            }
            _ => {}
        }

        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &mut self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut state.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

        let (content_state, toasts_state) = state.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let toasts = (!self.toasts.is_empty()).then(|| {
            overlay::Element::new(Box::new(Overlay {
                position: layout.bounds().position() + translation,
                toasts: &mut self.toasts,
                state: toasts_state,
                instants,
                on_close: &self.on_close,
                timeout_secs: self.timeout_secs,
            }))
        });
        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        (!overlays.is_empty()).then(move || overlay::Group::with_children(overlays).overlay())
    }
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    toasts: &'b mut [Element<'a, Message>],
    state: &'b mut [Tree],
    instants: &'b mut [Option<Instant>],
    on_close: &'b dyn Fn(usize) -> Message,
    timeout_secs: u64,
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            Fill,
            Fill,
            10.into(),
            10.0,
            Alignment::End,
            self.toasts,
            self.state,
        )
        .translate(Vector::new(self.position.x, self.position.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, state), layout) in self
            .toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.toasts
                .iter_mut()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            let mut next_redraw: Option<window::RedrawRequest> = None;

            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        let remaining = Duration::from_secs(self.timeout_secs)
                            .saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO {
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                            next_redraw = Some(window::RedrawRequest::NextFrame);
                        } else {
                            let redraw_at = window::RedrawRequest::At(*now + remaining);
                            next_redraw = next_redraw
                                .map(|redraw| redraw.min(redraw_at))
                                .or(Some(redraw_at));
                        }
                    }
                });

            if next_redraw.is_some() {
                shell.request_redraw();
            }
        }

        let viewport = layout.bounds();

        let _ = self
            .toasts
            .iter_mut()
            .zip(self.state.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
            .map(|(((child, state), layout), instant)| {
                let mut local_messages = vec![];
                let mut local_shell = Shell::new(&mut local_messages);

                child.as_widget_mut().update(
                    state,
                    event,
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    &mut local_shell,
                    &viewport,
                );

                if !local_shell.is_empty() {
                    instant.take();
                }

                shell.merge(local_shell, std::convert::identity);
            });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget().mouse_interaction(
                    state,
                    layout,
                    cursor,
                    &Rectangle::INFINITE,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }
}

impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(manager: Manager<'a, Message>) -> Self {
        Element::new(manager)
    }
}

fn styled(pair: theme::palette::Pair) -> container::Style {
    container::Style {
        background: Some(pair.color.into()),
        text_color: pair.text.into(),
        ..Default::default()
    }
}

fn primary(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    styled(palette.primary.weak)
}

fn secondary(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    styled(palette.secondary.weak)
}

fn success(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    styled(palette.success.weak)
}

fn danger(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    styled(palette.danger.weak)
}
