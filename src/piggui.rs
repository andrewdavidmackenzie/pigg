use clap::{Arg, ArgMatches};
use std::env;
use iced::widget::{container, Column, column, text_input, text, Button, Text};
use iced::{
    executor, window, Application, Command, Element, Event, Length, Settings, Subscription, Theme,
};
use modal::Modal;
use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw::config::HardwareConfig;
use crate::toast_handler::{ToastHandler, ToastMessage};
use crate::views::hardware_view::HardwareViewMessage::NewConfig;
use crate::views::hardware_view::{HardwareView, HardwareViewMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::main_row;
use crate::views::message_row::MessageRowMessage::ShowStatusMessage;
use crate::views::message_row::{MessageMessage, MessageRowMessage};
use crate::Message::*;
use views::pin_state::PinState;

mod file_helper;
#[cfg(any(feature = "fake_hw", feature = "pi_hw"))]
pub mod hardware_subscription;
mod hw;
pub mod network_subscription;
mod styles;
mod toast_handler;
mod views;
mod widgets;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: true,
        exit_on_close_request: false,
        size: LayoutSelector::get_default_window_size(),
        ..Default::default()
    };

    Piggui::run(Settings {
        window,
        ..Default::default()
    })
}

/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about("'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware");

    let app = app.arg(
        Arg::new("nodeid")
            .short('n')
            .long("nodeid")
            .num_args(1)
            .number_of_values(1)
            .value_name("NODEID")
            .help("Node Id of a piglet instance to connect to"),
    );

    let app = app.arg(
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}

/// These are the messages that Piggui responds to
#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(String, HardwareConfig),
    ConfigSaved,
    ConfigChangesMade,
    Save,
    Load,
    LayoutChanged(Layout),
    Hardware(HardwareViewMessage),
    Toast(ToastMessage),
    InfoRow(MessageRowMessage),
    WindowEvent(iced::Event),
    HardwareLost,
    MenuBarButtonClicked,
    ShowModal,
    HideModal,
    Submit,
    ConnectionId(String)
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    info_row: InfoRow,
    toast_handler: ToastHandler,
    hardware_view: HardwareView,
    show_modal: bool,
    connection_id: String,
}

async fn empty() {}
impl Piggui {
    fn hide_modal(&mut self) {
        self.show_modal = false;
        self.connection_id.clear();
    }
}
impl Application for Piggui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Piggui, Command<Message>) {
        let matches = get_matches();
        let config_filename = matches
            .get_one::<String>("config-file")
            .map(|s| s.to_string());

        let nodeid = matches.get_one::<String>("nodeid").map(|s| s.to_string());

        // TODO this will come from UI entry later. For now copy this from the output of piglet then run piggui
        //let node_id = "2r7vxyfvkfgwfkcxt5wky72jghy4n6boawnvz5fxes62tqmnnmhq";

        (
            Self {
                config_filename: None,
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                info_row: InfoRow::new(),
                toast_handler: ToastHandler::new(),
                hardware_view: HardwareView::new(nodeid),
                show_modal: false,
                connection_id: String::new(),
            },
            maybe_load_no_picker(config_filename),
        )
    }

    fn title(&self) -> String {
        self.config_filename
            .clone()
            .unwrap_or(String::from("Piggui"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            WindowEvent(event) => {
                if let iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) = event
                {
                    if self.unsaved_changes {
                        let _ = self
                            .toast_handler
                            .update(ToastMessage::UnsavedChangesExitToast, &self.hardware_view);
                        self.unsaved_changes = false;
                    } else {
                        return window::close(window::Id::MAIN);
                    }
                }
            }

            ConnectionId(connection_id) => {
                self.connection_id = connection_id;
                return Command::none();
            }
            Submit => {
                if !self.connection_id.is_empty() {
                    self.hide_modal();
                }

                return Command::none();
            }

            ShowModal => {
                self.show_modal = true;
                return iced::widget::focus_next();
            }
            HideModal => {
                self.hide_modal();
                return Command::none();
            }

            MenuBarButtonClicked => {
                return Command::none();
            }

            LayoutChanged(layout) => {
                // Keep overall window management at this level and out of LayoutSelector
                return window::resize(window::Id::MAIN, self.layout_selector.update(layout));
            }

            Save => return save(self.hardware_view.get_config()),

            ConfigSaved => {
                self.unsaved_changes = false;
                return Command::perform(empty(), |_| {
                    InfoRow(ShowStatusMessage(MessageMessage::Info(
                        "File saved successfully".to_string(),
                    )))
                });
            }

            Load => {
                if self.unsaved_changes {
                    let _ = self
                        .toast_handler
                        .update(ToastMessage::UnsavedChangesToast, &self.hardware_view);
                } else {
                    return Command::batch(vec![ToastHandler::clear_last_toast(), pick_and_load()]);
                }
            }

            Toast(toast_message) => {
                return self
                    .toast_handler
                    .update(toast_message, &self.hardware_view);
            }

            InfoRow(msg) => return self.info_row.update(msg),

            Hardware(msg) => return self.hardware_view.update(msg),

            ConfigChangesMade => self.unsaved_changes = true,

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                self.unsaved_changes = false;
                return Command::perform(empty(), |_| Hardware(NewConfig(config)));
            }
            HardwareLost => {
                return Command::perform(empty(), |_| {
                    InfoRow(ShowStatusMessage(MessageMessage::Error(
                        "Connection to Hardware Lost".to_string(),
                        "The connection to GPIO hardware has been lost. Check networking and try to re-connect".to_string()
                    )))
                });
            }
        }

        Command::none()
    }

    /*
       +-window-------------------------------------------------------------------------------+
       |  +-content(main_col)---------------------------------------------------------------+ |
       |  | +-main-row--------------------------------------------------------------------+ | |
       |  | |                                                                             | | |
       |  | |                                                                             | | |
       |  | |                                                                             | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  | +-info-row--------------------------------------------------------------------+ | |
       |  | |                                                                             | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  +---------------------------------------------------------------------------------+ |
       +--------------------------------------------------------------------------------------+
    */
    fn view(&self) -> Element<Message> {
        let main_col = Column::new()
            .push(main_row::view(&self.hardware_view, &self.layout_selector))
            .push(
                self.info_row
                    .view(self.unsaved_changes, &self.hardware_view),
            );

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y();

        if self.show_modal {

            let modal = container(
                column![
                    text("Connect to Remote Pi").size(24),
                    column![
                        column![
                            text("Connection id").size(12),
                            text_input("Enter 10 digits connection id", &self.connection_id,)
                                .on_input(Message::ConnectionId)
                                .on_submit(Message::Submit)
                                .padding(5),
                        ]
                        .spacing(5),
                        Button::new(Text::new("Connect")).on_press(Message::HideModal)
                    ]
                    .spacing(10)
                ]
                    .spacing(20),
            )
                .width(300)
                .padding(10);

            Modal::new(content, modal)
                .on_blur(Message::HideModal)
                .into()
        } else {

            self.toast_handler.view(content.into())
        }

    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Subscribe to events from Hardware, from Windows and timings for StatusRow
    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            iced::event::listen().map(WindowEvent),
            self.info_row.subscription().map(InfoRow),
            self.hardware_view.subscription().map(Hardware),
        ];

        Subscription::batch(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_close_with_unsaved_changes() {
        let mut app = Piggui::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a close window event
        let _ = app.update(WindowEvent(iced::Event::Window(
            window::Id::MAIN,
            window::Event::CloseRequested,
        )));

        // Check if a warning toast was added
        assert_eq!(app.toast_handler.get_toasts().len(), 1);
        let toast = &app.toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to exit without saving?"
        );
    }

    #[test]
    fn test_load_with_unsaved_changes() {
        let mut app = Piggui::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a load message
        let _ = app.update(Load);

        // Check if a warning toast was added
        assert_eq!(app.toast_handler.get_toasts().len(), 1);
        let toast = &app.toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to continue without saving?"
        );
    }
}
mod modal {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::advanced::{self, Clipboard, Shell};
    use iced::alignment::Alignment;
    use iced::event;
    use iced::mouse;
    use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Vector};

    /// A widget that centers a modal element over some base element
    pub struct Modal<'a, Message, Theme, Renderer> {
        base: Element<'a, Message, Theme, Renderer>,
        modal: Element<'a, Message, Theme, Renderer>,
        on_blur: Option<Message>,
    }

    impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer> {
        /// Returns a new [`Modal`]
        pub fn new(
            base: impl Into<Element<'a, Message, Theme, Renderer>>,
            modal: impl Into<Element<'a, Message, Theme, Renderer>>,
        ) -> Self {
            Self {
                base: base.into(),
                modal: modal.into(),
                on_blur: None,
            }
        }

        /// Sets the message that will be produces when the background
        /// of the [`Modal`] is pressed
        pub fn on_blur(self, on_blur: Message) -> Self {
            Self {
                on_blur: Some(on_blur),
                ..self
            }
        }
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Modal<'a, Message, Theme, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn children(&self) -> Vec<widget::Tree> {
            vec![
                widget::Tree::new(&self.base),
                widget::Tree::new(&self.modal),
            ]
        }

        fn diff(&self, tree: &mut widget::Tree) {
            tree.diff_children(&[&self.base, &self.modal]);
        }

        fn size(&self) -> Size<Length> {
            self.base.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.base.as_widget().layout(
                &mut tree.children[0],
                renderer,
                limits,
            )
        }

        fn on_event(
            &mut self,
            state: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.base.as_widget_mut().on_event(
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

        fn draw(
            &self,
            state: &widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.base.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut widget::Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
            Some(overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                content: &mut self.modal,
                tree: &mut state.children[1],
                size: layout.bounds().size(),
                on_blur: self.on_blur.clone(),
            })))
        }

        fn mouse_interaction(
            &self,
            state: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.base.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn operate(
            &self,
            state: &mut widget::Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.base.as_widget().operate(
                &mut state.children[0],
                layout,
                renderer,
                operation,
            );
        }
    }

    struct Overlay<'a, 'b, Message, Theme, Renderer> {
        position: Point,
        content: &'b mut Element<'a, Message, Theme, Renderer>,
        tree: &'b mut widget::Tree,
        size: Size,
        on_blur: Option<Message>,
    }

    impl<'a, 'b, Message, Theme, Renderer>
    overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Message, Theme, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn layout(
            &mut self,
            renderer: &Renderer,
            _bounds: Size,
        ) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, self.size)
                .width(Length::Fill)
                .height(Length::Fill);

            let child = self
                .content
                .as_widget()
                .layout(self.tree, renderer, &limits)
                .align(Alignment::Center, Alignment::Center, limits.max());

            layout::Node::with_children(self.size, vec![child])
                .move_to(self.position)
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            let content_bounds = layout.children().next().unwrap().bounds();

            if let Some(message) = self.on_blur.as_ref() {
                if let Event::Mouse(mouse::Event::ButtonPressed(
                                        mouse::Button::Left,
                                    )) = &event
                {
                    if !cursor.is_over(content_bounds) {
                        shell.publish(message.clone());
                        return event::Status::Captured;
                    }
                }
            }

            self.content.as_widget_mut().on_event(
                self.tree,
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            )
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..renderer::Quad::default()
                },
                Color {
                    a: 0.80,
                    ..Color::BLACK
                },
            );

            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &layout.bounds(),
            );
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.content.as_widget().operate(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'c>(
            &'c mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
            self.content.as_widget_mut().overlay(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                Vector::ZERO,
            )
        }
    }

    impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
    where
        Theme: 'a,
        Message: 'a + Clone,
        Renderer: 'a + advanced::Renderer,
    {
        fn from(modal: Modal<'a, Message, Theme, Renderer>) -> Self {
            Element::new(modal)
        }
    }
}