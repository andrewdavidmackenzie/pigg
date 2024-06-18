use std::env;

use iced::widget::{container, Column};
use iced::{
    executor, window, Application, Command, Element, Length, Settings, Subscription, Theme,
};

use widgets::toast::{self, Manager, Status, Toast};

use crate::file_helper::{load, load_via_picker, save};
use crate::hw::GPIOConfig;
use crate::pin_state::PinState;
use crate::views::hardware_button::hw_description;
use crate::views::hardware_view::HardwareMessage::NewConfig;
use crate::views::hardware_view::{HardwareMessage, HardwareView};
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::status_row::StatusMessage::{Error, Info};
use crate::views::status_row::StatusRowMessage::ShowStatusMessage;
use crate::views::status_row::{StatusRow, StatusRowMessage};
use crate::views::version::version;
use crate::views::{info_row, main_row};
use crate::Message::*;

mod file_helper;
mod hw;
mod pin_state;
mod styles;
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

#[derive(Debug, Clone)]
pub enum ToastMessage {
    VersionToast,
    HardwareDetailsToast,
    Close(usize),
    Timeout(f64),
}

/// These are the messages that Piggui responds to
#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(String, GPIOConfig),
    LayoutChanged(Layout),
    ConfigChangesMade,
    Hardware(HardwareMessage),
    Toast(ToastMessage),
    Save,
    Load,
    SaveCancelled,
    StatusRow(StatusRowMessage),
    WindowEvent(iced::Event),
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    pub toasts: Vec<Toast>,
    pub showing_toast: bool,
    timeout_secs: u64,
    unsaved_changes: bool,
    pending_load: bool,
    status_row: StatusRow,
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
                toasts: Vec::new(),
                showing_toast: false,
                timeout_secs: toast::DEFAULT_TIMEOUT,
                unsaved_changes: false,
                pending_load: false,
                status_row: StatusRow::new(),
                hardware_view: HardwareView::new(),
            },
            match env::args().nth(1) {
                Some(filename) => Command::perform(load(filename), |result| match result {
                    Ok((filename, config)) => ConfigLoaded(filename, config),
                    Err(e) => Message::StatusRow(ShowStatusMessage(Error(
                        "Error loading config from file".into(),
                        format!("Error loading the file specified on command line: {}", e),
                    ))),
                }),
                None => Command::none(),
            },
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
                        self.toasts.clear();
                        self.toasts.push(Toast {
                            title: "Unsaved Changes".into(),
                            body: "You have unsaved changes. Do you want to exit without saving?"
                                .into(),
                            status: Status::Danger,
                        });
                        self.showing_toast = true;
                        self.unsaved_changes = false;
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

            SaveCancelled => {
                // TODO this is not always true no? If you don't have unsaved changes, you can still
                // Save, then cancel it... it should push/pop the previous value I think?
                self.unsaved_changes = true;
            }

            Load => {
                if !self.showing_toast {
                    // Add a new toast if `show_toast` is false
                    if self.unsaved_changes {
                        self.toasts.clear();
                        self.toasts.push(Toast {
                            title: "Unsaved Changes".into(),
                            body:
                                "You have unsaved changes. Do you want to continue without saving?"
                                    .into(),
                            status: Status::Danger,
                        });
                        self.showing_toast = true;
                        self.pending_load = true;
                    } else {
                        return Command::perform(load_via_picker(), |result| match result {
                            Ok(Some((filename, config))) => ConfigLoaded(filename, config),
                            Ok(None) => {
                                StatusRow(ShowStatusMessage(Info("File load cancelled".into())))
                            }
                            Err(e) => StatusRow(ShowStatusMessage(Error(
                                "File could not be loaded".into(),
                                format!("Error loading file: {e}"),
                            ))),
                        });
                    }
                } else {
                    // Close the existing toast if `show_toast` is true
                    let index = self.toasts.len() - 1;
                    return Command::perform(empty(), move |_| {
                        Message::Toast(ToastMessage::Close(index))
                    });
                }
            }

            Message::Toast(toast_message) => match toast_message {
                ToastMessage::VersionToast => {
                    self.toasts.clear();
                    self.toasts.push(Toast {
                        title: "About Piggui".into(),
                        body: version(),
                        status: Status::Primary,
                    });
                    self.showing_toast = true;
                }
                ToastMessage::HardwareDetailsToast => {
                    if self.showing_toast {
                        // Close the existing toast if `show_toast` is true
                        let index = self.toasts.len() - 1;
                        return Command::perform(empty(), move |_| {
                            Message::Toast(ToastMessage::Close(index))
                        });
                    } else {
                        self.toasts.clear();
                        self.toasts.push(Toast {
                            title: "About Connected Hardware".into(),
                            body: hw_description(&self.hardware_view),
                            status: Status::Primary,
                        });
                        self.showing_toast = true;
                    }
                }
                ToastMessage::Close(index) => {
                    self.showing_toast = false;
                    self.toasts.remove(index);
                    if self.pending_load {
                        self.pending_load = false;
                        return Command::perform(load_via_picker(), |result| match result {
                            Ok(Some((filename, config))) => ConfigLoaded(filename, config),
                            Ok(None) => {
                                StatusRow(ShowStatusMessage(Info("File load cancelled".into())))
                            }
                            Err(e) => StatusRow(ShowStatusMessage(Error(
                                "File could not be loaded".into(),
                                format!("Error loading file: {e}"),
                            ))),
                        });
                    }
                }
                ToastMessage::Timeout(timeout) => {
                    self.timeout_secs = timeout as u64;
                }
            },

            Message::StatusRow(msg) => return self.status_row.update(msg),

            Hardware(msg) => return self.hardware_view.update(msg),

            ConfigChangesMade => self.unsaved_changes = true,

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                return Command::perform(empty(), |_| Hardware(NewConfig(config)));
            }
        }

        Command::none()
    }

    /*
       +-window-------------------------------------------------------------------------------+
       |  +-content(main_col)---------------------------------------------------------------+ |
       |  | +-main-row--------------------------------------------------------------------+ | |
       |  | | +-configuration-column-+--------------------------------------------------+ | | |
       |  | | |                      |                                                  | | | |
       |  | | +-configuration-column-+--------------------------------------------------+ | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  | +-info-row--------------------------------------------------------------------+ | |
       |  | |  <version> | <hardware> | <status>                                          | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  +---------------------------------------------------------------------------------+ |
       +--------------------------------------------------------------------------------------+
    */
    fn view(&self) -> Element<Self::Message> {
        let main_col = Column::new()
            .push(main_row::view(self))
            .push(info_row::view(self));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y();

        Manager::new(content, &self.toasts, |index| {
            Message::Toast(ToastMessage::Close(index))
        })
        .timeout(self.timeout_secs)
        .into()
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
    async fn test_add_toast_message() {
        let mut app = Piggui::new(()).0;

        // No toasts should be present
        assert!(app.toasts.is_empty());

        // Add a toast
        let _ = app.update(Message::Toast(ToastMessage::VersionToast));

        // Check if a toast was added
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
        assert_eq!(toast.title, "About Piggui");
    }

    #[tokio::test]
    async fn test_close_toast_message() {
        let mut app = Piggui::new(()).0;

        // Add a toast
        let _ = app.update(Message::Toast(ToastMessage::VersionToast));

        // Ensure the toast was added
        assert_eq!(app.toasts.len(), 1);

        // Close the toast
        let _ = app.update(Message::Toast(ToastMessage::Close(0)));

        // Check if the toast was removed
        assert!(app.toasts.is_empty());
    }

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
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
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
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to continue without saving?"
        );
    }

    #[tokio::test]
    async fn test_toast_timeout() {
        let mut app = Piggui::new(()).0;

        // Send a timeout message
        let _ = app.update(Message::Toast(ToastMessage::Timeout(5.0)));

        // Check the timeout
        assert_eq!(app.timeout_secs, 5);
    }
}
