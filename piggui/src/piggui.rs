#![deny(clippy::unwrap_used)]

use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog::{
    ConnectDialog, ConnectDialogMessage, ConnectDialogMessage::HideConnectDialog,
};
use crate::views::hardware_view::{HardwareView, HardwareViewMessage};
use crate::views::info_dialog::{InfoDialog, InfoDialogMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_menu::{Layout, LayoutSelector};
#[cfg(not(target_arch = "wasm32"))]
use crate::views::message_box::InfoMessage;
use crate::views::message_box::InfoMessage::{Error, Info};
use crate::views::message_box::MessageRowMessage;
#[cfg(not(target_arch = "wasm32"))]
use crate::views::message_box::MessageRowMessage::ShowStatusMessage;
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialog;
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialogMessage;
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialogMessage::HideSsidDialog;
use crate::widgets::modal::modal;
use crate::Message::*;
#[cfg(not(target_arch = "wasm32"))]
use clap::{Arg, ArgMatches};
use iced::widget::{container, Column};
use iced::{window, Element, Length, Pixels, Settings, Subscription, Task, Theme};
#[cfg(all(feature = "iroh", not(target_arch = "wasm32")))]
use iroh::NodeId;
use pigdef::config::HardwareConfig;
#[cfg(feature = "usb")]
use pigdef::description::SerialNumber;
#[cfg(not(target_arch = "wasm32"))]
use piggpio::local_hardware;
#[cfg(feature = "discovery")]
use pignet::discovery::{DiscoveredDevice, DiscoveryEvent};
#[cfg(feature = "usb")]
use pignet::usb_host;
use pignet::HardwareConnection;
#[cfg(not(target_arch = "wasm32"))]
use pignet::HardwareConnection::Local;
use pignet::HardwareConnection::NoConnection;
#[cfg(feature = "discovery")]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::process;
#[cfg(all(any(feature = "iroh", feature = "tcp"), not(target_arch = "wasm32")))]
use std::str::FromStr;
use sysinfo::{Process, System};

#[cfg(feature = "discovery")]
mod discovery;
pub mod file_helper;
mod hardware_subscription;
#[cfg(not(target_arch = "wasm32"))]
mod local_host;
mod views;
mod widgets;

const PIGGUI_ID: &str = "piggui";
const CONNECTION_ERROR: &str = "Connection Error";
#[cfg(feature = "discovery")]
const DISCOVERY_ERROR: &str = "Discovery Error";

/// These are the messages that Piggui responds to
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    ConfigLoaded(String, HardwareConfig),
    ConfigSaved,
    ConfigChangesMade(bool),
    Save,
    Load,
    LayoutChanged(Layout),
    WindowSizeChangeRequest,
    Hardware(HardwareViewMessage),
    Modal(InfoDialogMessage),
    InfoRow(MessageRowMessage),
    WindowEvent(iced::Event),
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    ConnectDialog(ConnectDialogMessage),
    ConnectRequest(HardwareConnection),
    Connected,
    Disconnect,
    ConnectionError(String),
    MenuBarButtonClicked,
    #[cfg(feature = "discovery")]
    Discovery(DiscoveryEvent),
    #[cfg(feature = "usb")]
    SsidDialog(SsidDialogMessage),
    #[cfg(feature = "usb")]
    ResetSsid(String),
    #[cfg(feature = "usb")]
    SsidSpecSent(Result<(), String>),
}

/// [Piggui] holds the application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    info_row: InfoRow,
    modal_handler: InfoDialog,
    hardware_view: HardwareView,
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    connect_dialog: ConnectDialog,
    #[cfg(feature = "discovery")]
    discovered_devices: HashMap<String, DiscoveredDevice>,
    #[cfg(feature = "usb")]
    ssid_dialog: SsidDialog,
}

fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let settings = Settings {
        id: Some(PIGGUI_ID.into()),
        default_text_size: Pixels(14.0),
        ..Default::default()
    };

    iced::application(Piggui::title, Piggui::update, Piggui::view)
        .subscription(Piggui::subscription)
        .window_size((500.0, 800.0))
        .exit_on_close_request(false)
        .resizable(true)
        .settings(settings)
        .window_size(LayoutSelector::get_default_window_size())
        .theme(|_| Theme::Dark)
        .run_with(Piggui::new)
}

#[cfg(feature = "usb")]
#[allow(unused_variables)]
fn reset_ssid(serial_number: SerialNumber) -> Task<Message> {
    #[cfg(feature = "usb")]
    return Task::perform(usb_host::reset_ssid_spec(serial_number), |res| match res {
        Ok(_) => InfoRow(ShowStatusMessage(Info(
            "Wi-Fi Setup reset to Default by USB".into(),
        ))),
        Err(e) => InfoRow(ShowStatusMessage(Error(
            "Error resetting Wi-Fi Setup via USB".into(),
            e.to_string(),
        ))),
    });
    #[cfg(not(feature = "usb"))]
    Task::none()
}

#[cfg(not(target_arch = "wasm32"))]
/// Check that there is no pigglet running on the same device
fn process_running(process_name: &str) -> bool {
    let my_pid = process::id();
    let sys = System::new_all();
    let instances: Vec<&Process> = sys
        .processes_by_exact_name(process_name.as_ref())
        .filter(|p| p.thread_kind().is_none() && p.pid().as_u32() != my_pid)
        .collect();
    !instances.is_empty()
}

impl Piggui {
    /// Disconnect from the hardware
    fn disconnect(&mut self) {
        self.info_row.clear_info_messages(); // Clear out-of-date messages
        self.info_row
            .add_info_message(Info("Disconnected".to_string()));
        self.config_filename = None;
        self.unsaved_changes = false;
        self.hardware_view.new_connection(NoConnection);
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn empty() {}

    fn new() -> (Self, Task<Message>) {
        #[cfg(not(target_arch = "wasm32"))]
        let matches = get_matches();
        #[cfg(not(target_arch = "wasm32"))]
        let config_filename = matches.get_one::<String>("config").map(|s| s.to_string());
        #[cfg(target_arch = "wasm32")]
        let config_filename = None;

        // We may request a number of tasks to be done on start
        #[allow(unused_mut)]
        let mut tasks = vec![maybe_load_no_picker(config_filename.clone())];

        // If there is an instance of pigglet running and in control of any local GPIO
        // hardware, then we will not offer the option to directly control the local GPIO.
        // We will rely on discovery methods to provide an option on the GUI to connect to the
        // locally running instance of pigglet and hence be able to control the GPIO from the GUI.
        // The same if there is another instance of piggui running and able to connect to hardware
        #[cfg(not(target_arch = "wasm32"))]
        let local_hardware_opt = if process_running("pigglet") {
            let message = Task::perform(Self::empty(), |_| {
                let string =
                    "GPIO Hardware is being controlled by an instance of pigglet".to_string();
                println!("{string}");
                InfoRow(ShowStatusMessage(InfoMessage::Warning(string)))
            });
            tasks.push(message);
            None
        } else if process_running("piggui") {
            let message = Task::perform(Self::empty(), |_| {
                let string =
                    "GPIO Hardware is being controlled by another instance of piggui".to_string();
                println!("{string}");
                InfoRow(ShowStatusMessage(InfoMessage::Warning(string)))
            });
            tasks.push(message);
            None
        } else {
            local_hardware()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let default_connection = match &local_hardware_opt {
            None => NoConnection,
            Some(_hw) => Local,
        };

        #[cfg(feature = "discovery")]
        let discovered_devices = discovery::local_discovery(local_hardware_opt);

        (
            Self {
                config_filename,
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                info_row: InfoRow::new(),
                modal_handler: InfoDialog::new(),
                #[cfg(not(target_arch = "wasm32"))]
                hardware_view: HardwareView::new(choose_hardware_connection(
                    &matches,
                    default_connection,
                )),
                #[cfg(target_arch = "wasm32")]
                hardware_view: HardwareView::new(HardwareConnection::default()),
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                connect_dialog: ConnectDialog::new(),
                #[cfg(feature = "discovery")]
                discovered_devices,
                #[cfg(feature = "usb")]
                ssid_dialog: SsidDialog::new(),
            },
            Task::batch(tasks),
        )
    }

    fn title(&self) -> String {
        self.config_filename
            .clone()
            .unwrap_or(String::from("piggui"))
    }

    fn window_size_change_request(&self) -> Task<Message> {
        let layout_size = self.layout_selector.window_size_requested(
            self.hardware_view.get_description(),
            self.hardware_view.get_config(),
        );
        window::get_latest().then(move |latest| {
            if let Some(id) = latest {
                window::resize(id, layout_size)
            } else {
                Task::none()
            }
        })
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            WindowEvent(event) => {
                if let iced::Event::Window(window::Event::CloseRequested) = event {
                    if self.unsaved_changes {
                        let _ = self
                            .modal_handler
                            .update(InfoDialogMessage::UnsavedChangesExitModal);
                    } else {
                        return window::get_latest().and_then(window::close);
                    }
                }
            }

            LayoutChanged(new_layout) => {
                self.layout_selector.update(new_layout);
                return self.window_size_change_request();
            }

            WindowSizeChangeRequest => {
                return self.window_size_change_request();
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
                        .update(InfoDialogMessage::UnsavedLoadConfigChangesModal);
                } else {
                    return pick_and_load();
                }
            }

            Modal(modal_message) => {
                return self.modal_handler.update(modal_message);
            }

            #[cfg(any(feature = "iroh", feature = "tcp"))]
            ConnectDialog(connect_dialog_message) => {
                return self.connect_dialog.update(connect_dialog_message);
            }

            InfoRow(msg) => {
                return self.info_row.update(msg);
            }

            Hardware(msg) => {
                return self.hardware_view.update(msg);
            }

            ConfigChangesMade(resize_window) => {
                self.unsaved_changes = true;
                if resize_window {
                    return self.window_size_change_request();
                }
            }

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                self.unsaved_changes = false;
                self.hardware_view.new_config(config);
            }

            ConnectRequest(new_connection) => {
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.disable_widgets_and_load_spinner();
                self.hardware_view.new_connection(new_connection);
            }

            Connected => {
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.hide_modal();
                self.info_row.clear_info_messages(); // Hide out-of-date messages
                self.info_row
                    .add_info_message(Info("Connected".to_string()));
                #[cfg(debug_assertions)] // Output used in testing - DON'T REMOVE
                println!("Connected to hardware");
                return self.window_size_change_request();
            }

            ConnectionError(details) => {
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                self.hardware_view.new_connection(NoConnection);
                self.info_row
                    .add_info_message(Error(CONNECTION_ERROR.to_string(), details.clone()));
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.set_error(details);
            }

            MenuBarButtonClicked => { /* Needed for Highlighting on hover to work on the menu bar */
            }

            #[cfg(feature = "discovery")]
            Discovery(event) => self.discovery_event(event),

            #[cfg(feature = "usb")]
            SsidDialog(ssid_dialog_message) => {
                return self.ssid_dialog.update(ssid_dialog_message);
            }

            #[cfg(feature = "usb")]
            ResetSsid(serial_number) => {
                return reset_ssid(serial_number);
            }

            #[cfg(feature = "usb")]
            SsidSpecSent(result) => match result {
                Ok(_) => {
                    self.ssid_dialog.hide_modal();
                    self.info_row
                        .add_info_message(Info("Wi-Fi Setup sent via USB".to_string()));
                }
                Err(e) => {
                    self.ssid_dialog.enable_widgets_and_hide_spinner();
                    self.ssid_dialog.set_error(e.clone());
                    self.info_row.add_info_message(Error(
                        "Error sending Wi-Fi Setup via USB".to_string(),
                        e,
                    ));
                }
            },
            Disconnect => self.disconnect(),
        }

        Task::none()
    }

    /*
       +-window-------------------------------------------------------------------------------+
       |  +-content(main_col)---------------------------------------------------------------+ |
       |  | +-hardware-view---------------------------------------------------------------+ | |
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
    fn view(&self) -> Element<'_, Message> {
        let main_col = Column::new()
            .push(self.hardware_view.view(self.layout_selector.get()))
            .push(self.info_row.view(
                self.unsaved_changes,
                &self.layout_selector,
                &self.hardware_view,
                #[cfg(feature = "discovery")]
                &self.discovered_devices,
            ));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        #[cfg(any(feature = "iroh", feature = "tcp"))]
        if self.connect_dialog.show_modal {
            return modal(
                content,
                self.connect_dialog.view(),
                ConnectDialog(HideConnectDialog),
            );
        }

        #[cfg(feature = "usb")]
        if self.ssid_dialog.show_modal {
            return modal(content, self.ssid_dialog.view(), SsidDialog(HideSsidDialog));
        }

        if self.modal_handler.showing_modal() {
            return modal(
                content,
                self.modal_handler.view(),
                Modal(InfoDialogMessage::HideModal),
            );
        }

        content.into()
    }

    /// Subscribe to events from Hardware, from Windows and timings for StatusRow
    fn subscription(&self) -> Subscription<Message> {
        #[allow(unused_mut)]
        let mut subscriptions = vec![
            iced::event::listen().map(WindowEvent),
            self.modal_handler.subscription().map(Modal), // Handle Esc key event for modal
            self.info_row.subscription().map(InfoRow),
            self.hardware_view.subscription().map(Hardware),
        ];

        #[cfg(all(feature = "discovery", feature = "usb"))]
        subscriptions.push(Subscription::run(discovery::usb_discovery).map(Discovery));

        #[cfg(all(feature = "discovery", feature = "tcp"))]
        subscriptions.push(Subscription::run(discovery::mdns_discovery).map(Discovery));

        // Handle Keyboard events for ConnectDialog
        #[cfg(any(feature = "iroh", feature = "tcp"))]
        subscriptions.push(self.connect_dialog.subscription().map(ConnectDialog));

        #[cfg(feature = "usb")]
        subscriptions.push(self.ssid_dialog.subscription().map(SsidDialog));

        Subscription::batch(subscriptions)
    }

    #[cfg(feature = "discovery")]
    /// Process [DiscoveryEvent] messages related to discovery/loss of devices
    fn discovery_event(&mut self, event: DiscoveryEvent) {
        match event {
            DiscoveryEvent::DeviceFound(serial_number, discovered_device) => {
                let method = discovered_device.discovery_method.clone();
                self.info_row
                    .add_info_message(Info(format!("Device Found via {method}")));
                // if the device is already in the list of discovered devices, make sure this method
                // exists in the set of methods that can be used to connect to it
                if let Some(known_device) = self.discovered_devices.get_mut(&serial_number) {
                    known_device
                        .hardware_connections
                        .extend(discovered_device.hardware_connections);
                } else {
                    // new device, add to the map
                    self.discovered_devices
                        .insert(serial_number, discovered_device);
                }
            }
            DiscoveryEvent::DeviceLost(key, _method) => {
                // TODO only remove if not also discovered by some other method?
                // TODO WIll need changes in discovery of device and how stored in discovered_devices too
                if self.discovered_devices.remove(&key).is_some() {
                    self.info_row
                        .add_info_message(Info("Device Lost".to_string()));
                }
            }
            DiscoveryEvent::DeviceError(e) => {
                self.info_row
                    .add_info_message(Error(DISCOVERY_ERROR.to_string(), e.clone()));
            }
            DiscoveryEvent::Error(e) => {
                self.info_row
                    .add_info_message(Error(DISCOVERY_ERROR.to_string(), e.clone()));
            }
            #[cfg(target_os = "linux")]
            DiscoveryEvent::USBPermissionsError(_) => {
                // SHow the dialog explaining the error
                let _ = self.modal_handler.update(InfoDialogMessage::ErrorWithHelp("USB Permissions Error",
                           "Your user lacks the required permissions on USB device folders and files to write \
                to USB. Please consult the help at the link below on how to fix it",
                           "https://github.com/andrewdavidmackenzie/pigg/blob/master/HELP.md#permission-denied-os-error-13-linux-only"));
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Determine the hardware connection based on command line options
#[allow(unused_variables)]
fn choose_hardware_connection(
    matches: &ArgMatches,
    default_connection: HardwareConnection,
) -> HardwareConnection {
    #[allow(unused_mut)]
    let mut connection = default_connection;

    #[cfg(feature = "iroh")]
    if let Some(node_str) = matches.get_one::<String>("nodeid") {
        if let Ok(nodeid) = NodeId::from_str(node_str) {
            connection = HardwareConnection::Iroh(nodeid, None);
        } else {
            eprintln!("Could not create a NodeId for IrohNet from '{node_str}'");
        }
    }

    #[cfg(feature = "tcp")]
    if let Some(ip_str) = matches.get_one::<String>("ip") {
        if let Ok(tcp_target) = parse_ip_string(ip_str) {
            connection = tcp_target;
        }
    }

    #[cfg(feature = "usb")]
    if let Some(usb_str) = matches.get_one::<String>("usb") {
        connection = HardwareConnection::Usb(usb_str.to_string());
    }

    connection
}

#[cfg(feature = "tcp")]
fn parse_ip_string(ip_str: &str) -> anyhow::Result<HardwareConnection> {
    let (ip_str, port_str) = ip_str
        .split_once(':')
        .ok_or(anyhow::anyhow!("Could not parse ip:port"))?;
    let ip = std::net::IpAddr::from_str(ip_str)?;
    let port = u16::from_str(port_str)?;
    Ok(HardwareConnection::Tcp(ip, port))
}

#[cfg(not(target_arch = "wasm32"))]
/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .num_args(1)
                .number_of_values(1)
                .value_name("Config File")
                .help("Path of a '.pigg' config file to load"),
        );

    #[cfg(feature = "iroh")]
    let app = app.arg(
        Arg::new("nodeid")
            .short('n')
            .long("nodeid")
            .num_args(1)
            .number_of_values(1)
            .value_name("NODEID")
            .help("Node Id of device to connect to via Iroh"),
    );

    #[cfg(feature = "tcp")]
    let app = app.arg(
        Arg::new("ip")
            .short('i')
            .long("ip")
            .num_args(1)
            .number_of_values(1)
            .value_name("IP")
            .help("'IP:port' of device to connect to via TCP"),
    );

    #[cfg(feature = "usb")]
    let app = app.arg(
        Arg::new("usb")
            .short('u')
            .long("usb")
            .num_args(1)
            .number_of_values(1)
            .value_name("Serial")
            .help("Serial Number of a device to connect to via USB"),
    );

    app.get_matches()
}
