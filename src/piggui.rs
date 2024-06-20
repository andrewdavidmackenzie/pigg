use std::env;

use iced::widget::{container, Column};
use iced::{
    executor, window, Application, Command, Element, Length, Settings, Subscription, Theme,
};

use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw::GPIOConfig;
use crate::toast_handler::{ToastHandler, ToastMessage};
use crate::views::hardware_view::HardwareMessage::NewConfig;
use crate::views::hardware_view::{HardwareMessage, HardwareView};
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::status_message::StatusRowMessage::ShowStatusMessage;
use crate::views::status_message::{StatusMessage, StatusRow, StatusRowMessage};
use crate::views::version::version;
use crate::views::{info_row, main_row};
use crate::Message::*;
use views::pin_state::PinState;

mod file_helper;
mod hw;
mod styles;
mod toast_handler;
mod views;
mod widgets;

fn main() -> Result<(), iced::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("{}", version());
        return Ok(());
    }

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

/// These are the messages that Piggui responds to
#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(String, GPIOConfig),
    ConfigSaved,
    LayoutChanged(Layout),
    ConfigChangesMade,
    Hardware(HardwareMessage),
    Toast(ToastMessage),
    Save,
    Load,
    StatusRow(StatusRowMessage),
    WindowEvent(iced::Event),
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    status_row: StatusRow,
    toast_handler: ToastHandler,
    hardware_view: HardwareView,
}

async fn empty() {}

impl Application for Piggui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Piggui, Command<Message>) {
        (
            Self {
                config_filename: None,
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                status_row: StatusRow::new(),
                toast_handler: ToastHandler::new(),
                hardware_view: HardwareView::new(),
            },
            maybe_load_no_picker(env::args().nth(1)),
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
                            .update(ToastMessage::UnsavedChangesExitToast, None);
                    } else {
                        return window::close(window::Id::MAIN);
                    }
                }
            }

            LayoutChanged(layout) => {
                // Keep overall window management at this level and out of LayoutSelector
                return window::resize(window::Id::MAIN, self.layout_selector.update(layout));
            }

            Save => {
                return save(self.hardware_view.get_config());
            }

            ConfigSaved => {
                self.unsaved_changes = false;
                return Command::perform(crate::empty(), |_| {
                    Message::StatusRow(ShowStatusMessage(StatusMessage::Info(
                        "File saved successfully".to_string(),
                    )))
                });
            }

            Load => {
                if !self.toast_handler.is_showing_toast() {
                    if self.unsaved_changes {
                        let _ = self
                            .toast_handler
                            .update(ToastMessage::UnsavedChangesToast, None);
                        self.toast_handler.set_pending_load(true);
                    } else {
                        return pick_and_load();
                    }
                } else if let Some(index) = self.toast_handler.get_latest_toast_index() {
                    return Command::perform(crate::empty(), move |_| {
                        Message::Toast(ToastMessage::Close(index))
                    });
                }
            }

            Toast(toast_message) => {
                return self
                    .toast_handler
                    .update(toast_message, Some(&self.hardware_view));
            }

            Message::StatusRow(msg) => return self.status_row.update(msg),

            Hardware(msg) => return self.hardware_view.update(msg),

            ConfigChangesMade => self.unsaved_changes = true,

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                self.unsaved_changes = false;
                return Command::perform(crate::empty(), |_| Hardware(NewConfig(config)));
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
            .push(main_row::view(self))
            .push(info_row::view(self));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y();

        self.toast_handler.view(content.into())
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Subscribe to events from Hardware, from Windows and timings for StatusRow
    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            self.hardware_view.subscription().map(Hardware),
            iced::event::listen().map(WindowEvent),
            self.status_row.subscription().map(Message::StatusRow),
        ];

        Subscription::batch(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_window_close_with_unsaved_changes() {
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

    #[tokio::test]
    async fn test_load_with_unsaved_changes() {
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

    #[tokio::test]
    async fn test_hardware_details_toast() {
        let mut app = Piggui::new(()).0;
        // Show hardware details toast
        let _ = app.update(Message::Toast(ToastMessage::HardwareDetailsToast));

        // Check if a toast was added
        assert_eq!(app.toast_handler.get_toasts().len(), 1);
        let toast = &app.toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "About Connected Hardware");
        assert!(!toast.body.is_empty());
    }
}
