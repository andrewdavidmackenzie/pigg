use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw::config::HardwareConfig;
use crate::modal_handler::{DisplayModal, ModalMessage};
use crate::views::hardware_view::{HardwareTarget, HardwareView, HardwareViewMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::main_row;
use crate::views::message_row::MessageMessage::Info;
use crate::views::message_row::{MessageMessage, MessageRowMessage};
use crate::widgets::modal::Modal;
use crate::Message::*;
#[cfg(not(target_arch = "wasm32"))]
use clap::{Arg, ArgMatches};
use iced::widget::{container, Column};
use iced::{
    executor, window, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use views::pin_state::PinState;

#[cfg(feature = "iroh")]
use crate::connect_dialog_handler::{
    ConnectDialog, ConnectDialogMessage, ConnectDialogMessage::HideConnectDialog,
};
#[cfg(feature = "iroh")]
use iroh_net::NodeId;
#[cfg(feature = "iroh")]
use std::str::FromStr;
#[cfg(feature = "iroh")]
pub mod connect_dialog_handler;
#[cfg(not(target_arch = "wasm32"))]
mod file_helper;
pub mod hardware_subscription;
mod hw;
mod modal_handler;
#[cfg(feature = "iroh")]
pub mod network_subscription;
mod styles;
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
    ModalHandle(ModalMessage),
    InfoRow(MessageRowMessage),
    WindowEvent(iced::Event),
    MenuBarButtonClicked,
    #[cfg(feature = "iroh")]
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
    modal_handler: DisplayModal,
    hardware_view: HardwareView,
    #[cfg(feature = "iroh")]
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

    #[cfg(feature = "iroh")]
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
        #[cfg(not(target_arch = "wasm32"))]
        let matches = get_matches();
        #[cfg(not(target_arch = "wasm32"))]
        let config_filename = matches
            .get_one::<String>("config-file")
            .map(|s| s.to_string());
        #[cfg(target_arch = "wasm32")]
        let config_filename = None;

        (
            Self {
                config_filename: config_filename.clone(),
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                info_row: InfoRow::new(),
                modal_handler: DisplayModal::new(),
                hardware_view: HardwareView::new(),
                #[cfg(feature = "iroh")]
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
                            .modal_handler
                            .update(ModalMessage::UnsavedChangesExitModal, &self.hardware_view);
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
                        .modal_handler
                        .update(ModalMessage::UnsavedChangesExitModal, &self.hardware_view);
                } else {
                    return Command::batch(vec![pick_and_load()]);
                }
            }

            ModalHandle(toast_message) => {
                return self
                    .modal_handler
                    .update(toast_message, &self.hardware_view);
            }

            #[cfg(feature = "iroh")]
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
                if new_target == HardwareTarget::NoHW {
                    #[cfg(feature = "iroh")]
                    self.connect_dialog.enable_widgets_and_hide_spinner();
                    self.info_connected("Disconnected from hardware".to_string());
                } else {
                    #[cfg(feature = "iroh")]
                    self.connect_dialog.disable_widgets_and_load_spinner();
                }
                self.hardware_target = new_target;
            }

            Connected => {
                #[cfg(feature = "iroh")]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                #[cfg(feature = "iroh")]
                self.connect_dialog.hide_modal();
                self.info_connected("Connected to hardware".to_string());
            }

            ConnectionError(message) => {
                #[cfg(feature = "iroh")]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                self.info_connection_error(message.clone());
                #[cfg(feature = "iroh")]
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

        #[cfg(feature = "iroh")]
        if self.connect_dialog.show_modal {
            return Modal::new(content, self.connect_dialog.view())
                .on_blur(Message::ConnectDialog(HideConnectDialog))
                .into();
        }

        if self.modal_handler.show_modal {
            return Modal::new(content, self.modal_handler.view())
                .on_blur(Message::ModalHandle(ModalMessage::HideModal))
                .into();
        }

        content.into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Subscribe to events from Hardware, from Windows and timings for StatusRow
    fn subscription(&self) -> Subscription<Message> {
        #[allow(unused_mut)]
        let mut subscriptions = vec![
            iced::event::listen().map(WindowEvent),
            self.modal_handler.subscription().map(ModalHandle), // Handle Esc key event for modal
            self.info_row.subscription().map(InfoRow),
            self.hardware_view
                .subscription(&self.hardware_target)
                .map(Hardware),
        ];

        // Handle Keyboard events for ConnectDialog
        #[cfg(feature = "iroh")]
        subscriptions.push(self.connect_dialog.subscription().map(ConnectDialog));

        Subscription::batch(subscriptions)
    }
}

/// Determine the hardware target based on command line options
#[allow(unused_variables)]
fn get_hardware_target(matches: &ArgMatches) -> HardwareTarget {
    #[allow(unused_mut)]
    let mut target = HardwareTarget::default();

    #[cfg(feature = "iroh")]
    if let Some(node_str) = matches.get_one::<String>("nodeid").map(|s| s.to_string()) {
        if let Ok(nodeid) = NodeId::from_str(&node_str) {
            target = HardwareTarget::Iroh(nodeid, None);
        } else {
            eprintln!("Could not create a NodeId for IrohNet from '{}'", node_str);
        }
    }

    target
}

#[cfg(not(target_arch = "wasm32"))]
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
