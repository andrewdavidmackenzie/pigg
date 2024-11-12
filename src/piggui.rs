use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw_definition::config::HardwareConfig;
use crate::views::hardware_view::{HardwareTarget, HardwareView, HardwareViewMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::message_box::MessageMessage::Info;
use crate::views::message_box::{MessageMessage, MessageRowMessage};
use crate::views::modal_handler::{DisplayModal, ModalMessage};
use crate::widgets::modal::modal;
use crate::Message::*;
#[cfg(not(target_arch = "wasm32"))]
use clap::{Arg, ArgMatches};
use iced::widget::{container, Column};
use iced::{window, Element, Length, Padding, Pixels, Settings, Subscription, Task, Theme};
use std::collections::HashMap;
use views::pin_state::PinState;

use crate::views::hardware_menu::{DeviceEvent, KnownDevice};

#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog_handler::{
    ConnectDialog, ConnectDialogMessage, ConnectDialogMessage::HideConnectDialog,
};
use crate::views::hardware_menu;
#[cfg(feature = "iroh")]
use iroh_net::NodeId;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::str::FromStr;

#[cfg(not(target_arch = "wasm32"))]
mod file_helper;
mod hardware_subscription;
mod hw;
mod hw_definition;
#[cfg(feature = "iroh")]
#[path = "networking/piggui_iroh_helper.rs"]
mod piggui_iroh_helper;
#[path = "networking/piggui_local_helper.rs"]
mod piggui_local_helper;
#[cfg(feature = "tcp")]
#[path = "networking/piggui_tcp_helper.rs"]
mod piggui_tcp_helper;
#[cfg(feature = "usb-raw")]
mod usb_raw;
mod views;
mod widgets;

const PIGGUI_ID: &str = "piggui";

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
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    ConnectDialog(ConnectDialogMessage),
    ConnectRequest(HardwareTarget),
    Connected,
    ConnectionError(String),
    MenuBarButtonClicked,
    Device(DeviceEvent),
    ConfigureWiFi(String),
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    info_row: InfoRow,
    modal_handler: DisplayModal,
    hardware_view: HardwareView,
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    connect_dialog: ConnectDialog,
    hardware_target: HardwareTarget,
    known_devices: HashMap<String, KnownDevice>,
}

fn main() -> iced::Result {
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

impl Piggui {
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    /// We have disconnected, or been disconnected from the hardware
    fn disconnected(&mut self) {
        self.info_row
            .add_info_message(Info("Disconnected from hardware".to_string()));
        self.config_filename = None;
        self.unsaved_changes = false;
        self.hardware_view = HardwareView::new()
    }

    fn new() -> (Self, Task<Message>) {
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
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                connect_dialog: ConnectDialog::new(),
                hardware_target: get_hardware_target(&matches),
                known_devices: HashMap::new(),
            },
            maybe_load_no_picker(config_filename),
        )
    }

    fn title(&self) -> String {
        self.config_filename
            .clone()
            .unwrap_or(String::from("piggui"))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            WindowEvent(event) => {
                if let iced::Event::Window(window::Event::CloseRequested) = event {
                    if self.unsaved_changes {
                        let _ = self
                            .modal_handler
                            .update(ModalMessage::UnsavedChangesExitModal, &self.hardware_view);
                    } else {
                        return window::get_latest().and_then(window::close);
                    }
                }
            }

            LayoutChanged(layout) => {
                let layout = self.layout_selector.update(layout);
                return window::get_latest().then(move |latest| {
                    if let Some(id) = latest {
                        window::resize(id, layout)
                    } else {
                        Task::none()
                    }
                });
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
                    let _ = self.modal_handler.update(
                        ModalMessage::UnsavedLoadConfigChangesModal,
                        &self.hardware_view,
                    );
                } else {
                    return Task::batch(vec![pick_and_load()]);
                }
            }

            ModalHandle(toast_message) => {
                return self
                    .modal_handler
                    .update(toast_message, &self.hardware_view);
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
                    #[cfg(any(feature = "iroh", feature = "tcp"))]
                    self.connect_dialog.enable_widgets_and_hide_spinner();
                    #[cfg(any(feature = "iroh", feature = "tcp"))]
                    self.disconnected();
                } else {
                    #[cfg(any(feature = "iroh", feature = "tcp"))]
                    self.connect_dialog.disable_widgets_and_load_spinner();
                }
                self.hardware_target = new_target;
            }

            Connected => {
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.hide_modal();
                self.info_row
                    .add_info_message(Info("Connected to hardware".to_string()));
                #[cfg(debug_assertions)] // Output used in testing - DON'T REMOVE
                println!("Connected to hardware");
            }

            ConnectionError(details) => {
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.enable_widgets_and_hide_spinner();
                self.info_row.add_info_message(MessageMessage::Error(
                    "Connection Error".to_string(),
                    details.clone(),
                ));
                #[cfg(any(feature = "iroh", feature = "tcp"))]
                self.connect_dialog.set_error(details);
            }

            MenuBarButtonClicked => { /* Needed for Highlighting on hover to work on menu bar */ }

            Device(event) => self.device_event(event),

            ConfigureWiFi(serial_number) => {
                println!(
                    "dialog to configure known device with serial number: {}",
                    serial_number
                )
            }
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
    fn view(&self) -> Element<Message> {
        let main_col = Column::new()
            .push(
                self.hardware_view
                    .view(self.layout_selector.get(), &self.hardware_target),
            )
            .push(self.info_row.view(
                self.unsaved_changes,
                &self.layout_selector,
                &self.hardware_view,
                &self.hardware_target,
                &self.known_devices,
            ));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(Padding::new(0.0))
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

        if self.modal_handler.show_modal {
            return modal(
                content,
                self.modal_handler.view(),
                ModalHandle(ModalMessage::HideModal),
            );
        }

        content.into()
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
            hardware_menu::subscription().map(Device),
        ];

        // Handle Keyboard events for ConnectDialog
        #[cfg(any(feature = "iroh", feature = "tcp"))]
        subscriptions.push(self.connect_dialog.subscription().map(ConnectDialog));

        Subscription::batch(subscriptions)
    }

    /// Process messages related to USB raw discovery of attached devices
    fn device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::DeviceFound(method, hardware_description, ssid_spec) => {
                self.info_row
                    .add_info_message(Info("USB Device Found".to_string()));
                self.known_devices.insert(
                    hardware_description.details.serial.clone(),
                    KnownDevice::Porky(method, hardware_description, ssid_spec),
                );
            }
            DeviceEvent::DeviceLost(hardware_description) => {
                self.info_row
                    .add_info_message(Info("USB Device Lost".to_string()));
                self.known_devices
                    .remove(&hardware_description.details.serial);
            }
            DeviceEvent::Error(e) => {
                self.info_row.add_info_message(MessageMessage::Error(
                    "Connection Error".to_string(),
                    e.clone(),
                ));
            }
        }
    }
}

/// Determine the hardware target based on command line options
#[allow(unused_variables)]
fn get_hardware_target(matches: &ArgMatches) -> HardwareTarget {
    #[allow(unused_mut)]
    let mut target = HardwareTarget::default();

    #[cfg(feature = "iroh")]
    if let Some(node_str) = matches.get_one::<String>("nodeid") {
        if let Ok(nodeid) = NodeId::from_str(node_str) {
            target = HardwareTarget::Iroh(nodeid, None);
        } else {
            eprintln!("Could not create a NodeId for IrohNet from '{}'", node_str);
        }
    }

    #[cfg(feature = "tcp")]
    if let Some(ip_str) = matches.get_one::<String>("ip") {
        if let Ok(tcp_target) = parse_ip_string(ip_str) {
            target = tcp_target;
        }
    }

    target
}

#[cfg(feature = "tcp")]
fn parse_ip_string(ip_str: &str) -> anyhow::Result<HardwareTarget> {
    let (ip_str, port_str) = ip_str
        .split_once(':')
        .ok_or(anyhow::anyhow!("Could not parse ip:port"))?;
    let ip = std::net::IpAddr::from_str(ip_str)?;
    let port = u16::from_str(port_str)?;
    Ok(HardwareTarget::Tcp(ip, port))
}

#[cfg(not(target_arch = "wasm32"))]
/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about("'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware");

    #[cfg(feature = "iroh")]
    let app = app.arg(
        Arg::new("nodeid")
            .short('n')
            .long("nodeid")
            .num_args(1)
            .number_of_values(1)
            .value_name("NODEID")
            .help("Node Id of a piglet instance to connect to"),
    );

    #[cfg(feature = "tcp")]
    let app = app.arg(
        Arg::new("ip")
            .short('i')
            .long("ip")
            .num_args(1)
            .number_of_values(1)
            .value_name("IP")
            .help("'IP:port' of a piglet instance to connect to"),
    );

    let app = app.arg(
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}
