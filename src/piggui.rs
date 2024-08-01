use crate::connect_dialog_handler::ConnectDialogMessage::HideConnectDialog;
use crate::connect_dialog_handler::{ConnectDialog, ConnectDialogMessage};
use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw::config::HardwareConfig;
use crate::toast_handler::{ToastHandler, ToastMessage};
use crate::views::hardware_view::{HardwareTarget, HardwareView, HardwareViewMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::main_row;
use crate::views::message_row::MessageMessage::Info;
use crate::views::message_row::{MessageMessage, MessageRowMessage};
use crate::widgets::modal::Modal;
use crate::Message::*;
use clap::{Arg, ArgMatches};
use iced::widget::{container, Column};
use iced::{
    executor, window, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use iroh_net::NodeId;
use std::str::FromStr;
use views::pin_state::PinState;

pub mod connect_dialog_handler;
#[cfg(feature = "files")]
mod file_helper;
pub mod hardware_subscription;
mod hw;
pub mod network_subscription;
mod styles;
mod toast_handler;
mod views;
mod widgets;

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
    MenuBarButtonClicked,
    ConnectDialog(ConnectDialogMessage),
    ConnectRequest(HardwareTarget),
    Connected,
    ConnectionError(String),
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    info_row: InfoRow,
    toast_handler: ToastHandler,
    hardware_view: HardwareView,
    connect_dialog: ConnectDialog,
    hardware_target: HardwareTarget,
}

impl Piggui {
    /// Send a connection error message to the Info Bar
    fn info_connection_error(&mut self, message: String) {
        self.info_row.add_info_message(
            MessageMessage::Error(
                "Connection Error".to_string(),
                format!("Error in connection to hardware: '{message}'. Check networking and try to re-connect")
            ));
    }

    /// Send a message about successful connection to the info bar
    fn info_connected(&mut self, message: String) {
        self.info_row.add_info_message(Info(message));
    }

    /// Send a connection error message to the connection dialog
    fn dialog_connection_error(&mut self, message: String) {
        self.connect_dialog.set_error(message);
    }
}

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

        (
            Self {
                config_filename: config_filename.clone(),
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                info_row: InfoRow::new(),
                toast_handler: ToastHandler::new(),
                hardware_view: HardwareView::new(),
                connect_dialog: ConnectDialog::new(),
                hardware_target: get_hardware_target(&matches),
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

            MenuBarButtonClicked => {
                return Command::none();
            }

            LayoutChanged(layout) => {
                return window::resize(window::Id::MAIN, self.layout_selector.update(layout));
            }

            Save => {
                return save(self.hardware_view.get_config());
            }

            ConfigSaved => {
                self.unsaved_changes = false;
                self.info_row
                    .add_info_message(Info("File saved successfully".to_string()));
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

            ConnectDialog(connect_dialog_message) => {
                return self.connect_dialog.update(connect_dialog_message);
            }

            InfoRow(msg) => {
                return self.info_row.update(msg);
            }

            Hardware(msg) => {
                return self.hardware_view.update(msg);
            }

            ConfigChangesMade => {
                self.unsaved_changes = true;
            }

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                self.unsaved_changes = false;
                self.hardware_view.new_config(config);
            }

            ConnectRequest(new_target) => {
                match new_target {
                    HardwareTarget::NoHW => {
                        self.connect_dialog.enable_widgets_and_hide_spinner();
                        self.info_connected("Disconnected from hardware".to_string());
                    }
                    _ => {
                        self.connect_dialog.disable_widgets_and_load_spinner();
                    }
                }
                self.hardware_target = new_target;
            }

            Connected => {
                self.connect_dialog.enable_widgets_and_hide_spinner();
                self.connect_dialog.hide_modal();
                self.info_connected("Connected to hardware".to_string());
            }

            ConnectionError(message) => {
                self.connect_dialog.enable_widgets_and_hide_spinner();
                self.info_connection_error(message.clone());
                self.dialog_connection_error(message);
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
            .push(main_row::view(
                &self.hardware_view,
                &self.layout_selector,
                &self.hardware_target,
            ))
            .push(self.info_row.view(
                self.unsaved_changes,
                &self.hardware_view,
                &self.hardware_target,
            ));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y();

        if self.connect_dialog.show_modal {
            Modal::new(content, self.connect_dialog.view())
                .on_blur(Message::ConnectDialog(HideConnectDialog))
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
            self.connect_dialog.subscription().map(ConnectDialog), // Handle Keyboard events for ConnectDialog
            self.info_row.subscription().map(InfoRow),
            self.hardware_view
                .subscription(&self.hardware_target)
                .map(Hardware),
        ];

        Subscription::batch(subscriptions)
    }
}

/// Determine the hardware target based on command line options
fn get_hardware_target(matches: &ArgMatches) -> HardwareTarget {
    let mut target = HardwareTarget::default();

    if let Some(node_str) = matches.get_one::<String>("nodeid").map(|s| s.to_string()) {
        if let Ok(nodeid) = NodeId::from_str(&node_str) {
            target = HardwareTarget::Remote(nodeid, None);
        } else {
            eprintln!("Could not create a NodeId for IrohNet from '{}'", node_str);
        }
    }

    target
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
