#[cfg(feature = "iroh")]
use self::ConnectDialogMessage::{
    ConnectButtonPressedIroh, DisplayIrohTab, NodeIdEntered, RelayURL,
};
#[cfg(feature = "tcp")]
use self::ConnectDialogMessage::{
    ConnectionButtonPressedTcp, DisplayTcpTab, IpAddressEntered, PortNumberEntered,
};
use self::ConnectDialogMessage::{
    ConnectionError, HideConnectDialog, ModalKeyEvent, ShowConnectDialog,
};
use crate::views::modal_handler::{
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_STYLE, MODAL_CONTAINER_STYLE,
};
#[cfg(feature = "tcp")]
use crate::HardwareTarget::Tcp;
#[cfg(feature = "tcp")]
use std::net::IpAddr;

use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
#[cfg(feature = "iroh")]
use crate::views::hardware_view::HardwareTarget::*;
use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;
use crate::Message;
use iced::keyboard::key;
#[allow(unused_imports)]
use iced::widget::{self, column, container, text, text_input, Button, Row, Text};
use iced::{keyboard, Color, Command, Element, Event, Length};
use iced_futures::Subscription;
#[cfg(feature = "iroh")]
use iroh_net::{relay::RelayUrl, NodeId};
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "iroh")]
const IROH_INFO_TEXT: &str = "To connect to a remote Pi using iroh-net, ensure piglet is running on the remote Pi. Retrieve the nodeid from piglet, enter it below, and optionally provide a Relay URL";
#[cfg(feature = "tcp")]
const TCP_INFO_TEXT: &str = "To connect to a remote Pi using TCP, ensure Pi is reachable over the network. Enter the device's IP address and the port number below.";

use std::sync::LazyLock;
static TCP_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
static IROH_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

const INFO_TEXT_STYLE: TextStyle = TextStyle {
    text_color: Color::from_rgba(0.8, 0.8, 0.8, 1.0), // Slightly grey color
};

const TEXT_BOX_CONTAINER_STYLE: ContainerStyle = ContainerStyle {
    border_color: Color::from_rgba(1.0, 1.0, 1.0, 0.8),
    background_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
    border_width: 2.0,
    border_radius: 10.0,
};

const CONNECTION_ERROR_DISPLAY: TextStyle = TextStyle {
    text_color: Color::from_rgba(0.8, 0.0, 0.0, 1.0),
};

const ACTIVE_TAB_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::BLACK,   // Black background for active tab
    text_color: Color::WHITE, // White text for contrast
    hovered_bg_color: Color::BLACK,
    hovered_text_color: Color::WHITE,
    border_radius: 4.0,
};

const INACTIVE_TAB_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT, // Transparent background for inactive tab
    text_color: Color::from_rgba(0.7, 0.7, 0.7, 1.0), // Gray text color to show it's inactive
    hovered_bg_color: Color::from_rgb(0.2, 0.2, 0.2), // Slightly darker gray when hovered
    hovered_text_color: Color::WHITE,
    border_radius: 4.0,
};

const TAB_BAR_STYLE: ContainerStyle = ContainerStyle {
    border_color: Color::TRANSPARENT,
    background_color: Color::from_rgb(0.2, 0.2, 0.2),
    border_width: 0.0,
    border_radius: 0.0,
};

#[derive(Debug, Clone)]
pub struct ConnectDialog {
    #[cfg(feature = "iroh")]
    nodeid: String,
    #[cfg(feature = "iroh")]
    relay_url: String,
    #[cfg(feature = "tcp")]
    ip_address: String,
    #[cfg(feature = "tcp")]
    port_number: String,
    #[cfg(feature = "tcp")]
    tcp_connection_error: String,
    #[cfg(feature = "iroh")]
    iroh_connection_error: String,
    pub show_modal: bool,
    show_spinner: bool,
    disable_widgets: bool,
    display_iroh: bool,
}

#[derive(Clone, Debug)]
pub enum ConnectDialogMessage {
    #[cfg(feature = "iroh")]
    NodeIdEntered(String),
    #[cfg(feature = "iroh")]
    RelayURL(String),
    ModalKeyEvent(Event),
    #[cfg(feature = "iroh")]
    ConnectButtonPressedIroh(String, String),
    #[cfg(feature = "tcp")]
    ConnectionButtonPressedTcp(String, String),
    HideConnectDialog,
    ShowConnectDialog,
    ConnectionError(String),
    #[cfg(feature = "iroh")]
    DisplayIrohTab,
    #[cfg(feature = "tcp")]
    DisplayTcpTab,
    #[cfg(feature = "tcp")]
    IpAddressEntered(String),
    #[cfg(feature = "tcp")]
    PortNumberEntered(String),
}
impl Default for ConnectDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectDialog {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "iroh")]
            nodeid: String::new(),
            #[cfg(feature = "iroh")]
            relay_url: String::new(),
            #[cfg(feature = "iroh")]
            iroh_connection_error: String::new(),
            #[cfg(feature = "tcp")]
            tcp_connection_error: String::new(),
            show_modal: false,
            show_spinner: false,
            disable_widgets: false,
            display_iroh: true,
            #[cfg(feature = "tcp")]
            ip_address: String::new(),
            #[cfg(feature = "tcp")]
            port_number: String::new(),
        }
    }

    /// Set the error state of the dialog with a message to display
    pub fn set_error(&mut self, error: String) {
        #[cfg(feature = "iroh")]
        {
            self.iroh_connection_error = error.clone();
        }
        #[cfg(feature = "tcp")]
        {
            self.tcp_connection_error = error.clone();
        }
    }

    async fn empty() {}

    pub fn update(&mut self, message: ConnectDialogMessage) -> Command<Message> {
        match message {
            #[cfg(feature = "tcp")]
            ConnectionButtonPressedTcp(ip_address, port_num) => {
                // Display error when Ip address field is empty
                if ip_address.trim().is_empty() {
                    self.tcp_connection_error = String::from("Please Enter IP Address");
                    return Command::none();
                }

                // Display error when port number field is empty
                if port_num.trim().is_empty() {
                    self.tcp_connection_error = String::from("Please Enter Port Number");
                    return Command::none();
                }

                // Validate IP address
                return match IpAddr::from_str(ip_address.as_str().trim()) {
                    Ok(ip) => {
                        // Validate port number
                        match port_num.trim().parse::<u16>() {
                            Ok(port) => {
                                self.tcp_connection_error.clear();

                                // Proceed to request connection when the port number is valid
                                Command::perform(Self::empty(), move |_| {
                                    Message::ConnectRequest(Tcp(ip, port))
                                })
                            }
                            Err(e) => {
                                self.tcp_connection_error = format!("Invalid Port Number: {}", e);
                                self.show_spinner = false;
                                self.disable_widgets = false;
                                Command::none()
                            }
                        }
                    }
                    Err(err) => {
                        self.tcp_connection_error = format!("Invalid IP Address: {}", err);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Command::none()
                    }
                };
            }

            #[cfg(feature = "iroh")]
            ConnectButtonPressedIroh(node_id, url) => {
                if node_id.trim().is_empty() {
                    self.iroh_connection_error = String::from("Please Enter Node Id");
                    return Command::none();
                }

                return match NodeId::from_str(node_id.as_str().trim()) {
                    Ok(nodeid) => {
                        let url_str = url.trim();
                        let relay_url = if url_str.is_empty() {
                            None
                        } else {
                            match RelayUrl::from_str(url_str) {
                                Ok(relay) => {
                                    self.iroh_connection_error.clear();
                                    Some(relay)
                                }
                                Err(err) => {
                                    self.show_spinner = false;
                                    self.disable_widgets = false;
                                    self.iroh_connection_error = format!("{}", err);
                                    return Command::none();
                                }
                            }
                        };

                        Command::perform(Self::empty(), move |_| {
                            Message::ConnectRequest(Iroh(nodeid, relay_url))
                        })
                    }
                    Err(err) => {
                        self.iroh_connection_error = format!("{}", err);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Command::none()
                    }
                };
            }

            #[cfg(feature = "tcp")]
            DisplayTcpTab => {
                #[cfg(feature = "iroh")]
                {
                    self.display_iroh = false;
                }
                #[cfg(feature = "iroh")]
                self.iroh_connection_error.clear();
                text_input::focus(TCP_INPUT_ID.clone())
            }

            #[cfg(feature = "iroh")]
            DisplayIrohTab => {
                self.display_iroh = true;
                #[cfg(feature = "tcp")]
                self.tcp_connection_error.clear();
                text_input::focus(IROH_INPUT_ID.clone())
            }

            ShowConnectDialog => {
                self.show_modal = true;
                text_input::focus(IROH_INPUT_ID.clone())
            }

            HideConnectDialog => {
                self.hide_modal();
                Command::none()
            }

            ModalKeyEvent(event) => {
                match event {
                    // When Pressed `Tab` focuses on previous/next widget
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Tab),
                        modifiers,
                        ..
                    }) => {
                        if modifiers.shift() {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    // When Pressed `Esc` focuses on previous widget and hide modal
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Escape),
                        ..
                    }) => {
                        self.hide_modal();
                        Command::none()
                    }

                    _ => Command::none(),
                }
            }

            #[cfg(feature = "tcp")]
            IpAddressEntered(ip_addr) => {
                self.ip_address = ip_addr;
                Command::none()
            }

            #[cfg(feature = "tcp")]
            PortNumberEntered(port_num) => {
                self.port_number = port_num;
                Command::none()
            }

            #[cfg(feature = "iroh")]
            NodeIdEntered(node_id) => {
                self.nodeid = node_id;
                Command::none()
            }

            #[cfg(feature = "iroh")]
            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                Command::none()
            }

            ConnectionError(error) => {
                self.set_error(error);
                self.enable_widgets_and_hide_spinner();
                Command::none()
            }
        }
    }

    //noinspection RsLift
    pub fn view(&self) -> Element<'_, Message> {
        match (self.disable_widgets, self.display_iroh) {
            (true, true) => {
                #[cfg(feature = "iroh")]
                return self.create_iroh_container(false);
                #[cfg(not(feature = "iroh"))]
                return self.create_tcp_container(false);
            }
            (true, false) => {
                #[cfg(feature = "tcp")]
                return self.create_tcp_container(false);
                #[cfg(not(feature = "tcp"))]
                return self.create_iroh_container(false);
            }
            (false, false) => {
                #[cfg(feature = "tcp")]
                return self.create_tcp_container(true);
                #[cfg(not(feature = "tcp"))]
                return self.create_iroh_container(true);
            }
            (false, true) => {
                #[cfg(feature = "iroh")]
                return self.create_iroh_container(true);
                #[cfg(not(feature = "iroh"))]
                return self.create_tcp_container(true);
            }
        }
    }

    pub fn hide_modal(&mut self) {
        self.show_modal = false; // Hide the dialog
        #[cfg(feature = "iroh")]
        self.nodeid.clear(); // Clear the node id, on Cancel
        #[cfg(feature = "iroh")]
        self.iroh_connection_error.clear(); // Clear the error, on Cancel
        #[cfg(feature = "iroh")]
        self.relay_url.clear(); // Clear the relay url, on Cancel
        self.show_spinner = false; // Hide spinner, on Cancel
        self.disable_widgets = false; // Enable widgets, on Cancel
    }

    // Handle Keyboard events
    pub fn subscription(&self) -> Subscription<ConnectDialogMessage> {
        iced::event::listen().map(ModalKeyEvent)
    }

    pub fn disable_widgets_and_load_spinner(&mut self) {
        self.disable_widgets = true;
        self.show_spinner = true;
    }

    pub fn enable_widgets_and_hide_spinner(&mut self) {
        self.disable_widgets = false;
        self.show_spinner = false;
    }

    #[cfg(feature = "iroh")]
    fn create_connection_row_iroh(&self) -> Row<'_, Message> {
        if self.show_spinner && self.disable_widgets && self.display_iroh {
            self.create_spinner_row()
        } else {
            self.create_default_iroh_row()
        }
    }

    #[cfg(feature = "tcp")]
    fn create_connection_row_tcp(&self) -> Row<'_, Message> {
        if self.show_spinner && self.disable_widgets && !self.display_iroh {
            self.create_spinner_row()
        } else {
            self.create_default_tcp_row()
        }
    }

    fn create_spinner_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            )
            .push(
                Circular::new()
                    .easing(&EMPHASIZED_ACCELERATE)
                    .cycle_duration(Duration::from_secs_f32(2.0)),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
            )
            .spacing(160)
            .align_items(iced::Alignment::Center)
    }

    #[cfg(feature = "iroh")]
    fn create_default_iroh_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(HideConnectDialog))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(ConnectButtonPressedIroh(
                        self.nodeid.clone(),
                        self.relay_url.clone(),
                    )))
                    .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
            )
            .spacing(360)
            .align_items(iced::Alignment::Center)
    }

    #[cfg(feature = "tcp")]
    fn create_default_tcp_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(HideConnectDialog))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(ConnectionButtonPressedTcp(
                        self.ip_address.clone(),
                        self.port_number.clone(),
                    )))
                    .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
            )
            .spacing(360)
            .align_items(iced::Alignment::Center)
    }

    #[cfg(feature = "iroh")]
    fn create_text_container_iroh(&self) -> Element<'_, Message> {
        container(Text::new(IROH_INFO_TEXT).style(INFO_TEXT_STYLE.get_text_color()))
            .padding(10)
            .style(TEXT_BOX_CONTAINER_STYLE.get_container_style())
            .into()
    }

    #[cfg(feature = "tcp")]
    fn create_tcp_text_container(&self) -> Element<'_, Message> {
        container(Text::new(TCP_INFO_TEXT).style(INFO_TEXT_STYLE.get_text_color()))
            .padding(10)
            .style(TEXT_BOX_CONTAINER_STYLE.get_container_style())
            .into()
    }

    //noinspection RsLiveness
    #[cfg(feature = "iroh")]
    fn create_iroh_container(&self, input_enabled: bool) -> Element<'_, Message> {
        container(
            column![
                self.create_tab_buttons(true),
                column![
                    text("Connect To Remote Pi Using Iroh").size(20),
                    column![
                        self.create_text_container_iroh(),
                        text(self.iroh_connection_error.clone())
                            .style(CONNECTION_ERROR_DISPLAY.get_text_color()),
                        text("Node Id").size(12),
                        {
                            let mut node_input = text_input("Enter node id", &self.nodeid)
                                .padding(5)
                                .id(IROH_INPUT_ID.clone())
                                .on_submit(Message::ConnectDialog(
                                    ConnectDialogMessage::ConnectButtonPressedIroh(
                                        self.nodeid.clone(),
                                        self.relay_url.clone(),
                                    ),
                                ));
                            if input_enabled {
                                node_input = node_input.on_input(|input| {
                                    Message::ConnectDialog(ConnectDialogMessage::NodeIdEntered(
                                        input,
                                    ))
                                });
                            }
                            node_input
                        }
                    ]
                    .spacing(10),
                    column![text("Relay URL (Optional)").size(12), {
                        let mut relay_input =
                            text_input("Enter Relay Url (Optional)", &self.relay_url)
                                .padding(5)
                                .on_submit(Message::ConnectDialog(
                                    ConnectDialogMessage::ConnectButtonPressedIroh(
                                        self.nodeid.clone(),
                                        self.relay_url.clone(),
                                    ),
                                ));
                        if input_enabled {
                            relay_input = relay_input.on_input(|input| {
                                Message::ConnectDialog(ConnectDialogMessage::RelayURL(input))
                            });
                        }
                        relay_input
                    }]
                    .spacing(5),
                    self.create_connection_row_iroh(),
                ]
                .spacing(10)
            ]
            .spacing(20),
        )
        .style(MODAL_CONTAINER_STYLE.get_container_style())
        .width(520)
        .padding(15)
        .into()
    }

    //noinspection RsLiveness
    #[cfg(feature = "tcp")]
    fn create_tcp_container(&self, input_enabled: bool) -> Element<'_, Message> {
        container(
            column![
                self.create_tab_buttons(false),
                column![
                    text("Connect To Remote Pi Using Tcp").size(20),
                    column![
                        self.create_tcp_text_container(),
                        text(self.tcp_connection_error.clone())
                            .style(CONNECTION_ERROR_DISPLAY.get_text_color()),
                        text("IP Address").size(12),
                        {
                            let mut ip_input = text_input("Enter IP Address", &self.ip_address)
                                .padding(5)
                                .id(TCP_INPUT_ID.clone())
                                .on_submit(Message::ConnectDialog(
                                    ConnectDialogMessage::ConnectionButtonPressedTcp(
                                        self.ip_address.clone(),
                                        self.port_number.clone(),
                                    ),
                                ));
                            if input_enabled {
                                ip_input = ip_input.on_input(|input| {
                                    Message::ConnectDialog(ConnectDialogMessage::IpAddressEntered(
                                        input,
                                    ))
                                });
                            }
                            ip_input
                        }
                    ]
                    .spacing(10),
                    column![text("Port Number").size(12), {
                        let mut port_input = text_input("Enter Port Number", &self.port_number)
                            .padding(5)
                            .on_submit(Message::ConnectDialog(
                                ConnectDialogMessage::ConnectionButtonPressedTcp(
                                    self.ip_address.clone(),
                                    self.port_number.clone(),
                                ),
                            ));
                        if input_enabled {
                            port_input = port_input.on_input(|input| {
                                Message::ConnectDialog(ConnectDialogMessage::PortNumberEntered(
                                    input,
                                ))
                            });
                        }
                        port_input
                    }]
                    .spacing(5),
                    self.create_connection_row_tcp(),
                ]
                .spacing(10)
            ]
            .spacing(20),
        )
        .style(MODAL_CONTAINER_STYLE.get_container_style())
        .width(520)
        .padding(15)
        .into()
    }

    fn create_tab_buttons(&self, is_iroh_active: bool) -> Element<'_, Message> {
        #[allow(unused_variables)]
        let (iroh_style, tcp_style) = if is_iroh_active {
            (
                ACTIVE_TAB_BUTTON_STYLE.get_button_style(),
                INACTIVE_TAB_BUTTON_STYLE.get_button_style(),
            )
        } else {
            (
                INACTIVE_TAB_BUTTON_STYLE.get_button_style(),
                ACTIVE_TAB_BUTTON_STYLE.get_button_style(),
            )
        };

        let button_row = Row::new().spacing(5);

        #[cfg(feature = "iroh")]
        let button_row = button_row.push(
            Button::new(Text::new("Connect using Iroh").width(Length::Fill).size(22))
                .on_press(Message::ConnectDialog(DisplayIrohTab))
                .style(iroh_style)
                .width(Length::Fixed(260f32)),
        );

        #[cfg(feature = "tcp")]
        let button_row = button_row.push(
            Button::new(Text::new("Connect using TCP").width(Length::Fill).size(22))
                .on_press(Message::ConnectDialog(DisplayTcpTab))
                .style(tcp_style)
                .width(Length::Fixed(260f32)),
        );

        container(button_row)
            .style(TAB_BAR_STYLE.get_container_style())
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "tcp")]
    use crate::views::connect_dialog_handler::ConnectDialogMessage::ConnectionButtonPressedTcp;
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    #[test]
    fn test_show_connect_dialog() {
        let mut connect_dialog = ConnectDialog::new();
        assert!(!connect_dialog.show_modal);

        let _ = connect_dialog.update(ShowConnectDialog);
        assert!(connect_dialog.show_modal);
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_hide_connect_dialog() {
        let mut connect_dialog = ConnectDialog::new();
        connect_dialog.show_modal = true;

        let _ = connect_dialog.update(HideConnectDialog);
        assert!(!connect_dialog.show_modal);
        assert!(connect_dialog.nodeid.is_empty());
        assert!(connect_dialog.relay_url.is_empty());
        assert!(connect_dialog.iroh_connection_error.is_empty());
        assert!(!connect_dialog.show_spinner);
        assert!(!connect_dialog.disable_widgets);
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_node_id_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let node_id = "test_node_id".to_string();

        let _ = connect_dialog.update(NodeIdEntered(node_id.clone()));
        assert_eq!(connect_dialog.nodeid, node_id);
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_relay_url_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let relay_url = "test_relay_url".to_string();

        let _ = connect_dialog.update(RelayURL(relay_url.clone()));
        assert_eq!(connect_dialog.relay_url, relay_url);
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_connect_button_pressed_empty_node_id() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectButtonPressedIroh("".to_string(), "".to_string()));
        assert_eq!(connect_dialog.iroh_connection_error, "Please Enter Node Id");
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_connect_button_pressed_invalid_node_id() {
        let mut connect_dialog = ConnectDialog::new();
        let invalid_node_id = "invalid_node_id".to_string();

        let _ = connect_dialog.update(ConnectButtonPressedIroh(invalid_node_id, "".to_string()));
        assert!(!connect_dialog.iroh_connection_error.is_empty());
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn test_connection_error() {
        let mut connect_dialog = ConnectDialog::new();
        let error_message = "Connection failed".to_string();

        let _ = connect_dialog.update(ConnectionError(error_message.clone()));
        assert_eq!(connect_dialog.iroh_connection_error, error_message);
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_ip_address_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let ip_address = "192.168.1.1".to_string();

        let _ = connect_dialog.update(IpAddressEntered(ip_address.clone()));
        assert_eq!(connect_dialog.ip_address, ip_address);
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_port_number_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let port_number = "8080".to_string();

        let _ = connect_dialog.update(PortNumberEntered(port_number.clone()));
        assert_eq!(connect_dialog.port_number, port_number);
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_connection_button_pressed_tcp_empty_ip() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectionButtonPressedTcp(
            "".to_string(),
            "8080".to_string(),
        ));
        assert_eq!(
            connect_dialog.tcp_connection_error,
            "Please Enter IP Address"
        );
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_connection_button_pressed_tcp_empty_port() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectionButtonPressedTcp(
            "192.168.1.1".to_string(),
            "".to_string(),
        ));
        assert_eq!(
            connect_dialog.tcp_connection_error,
            "Please Enter Port Number"
        );
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_connection_button_pressed_tcp_invalid_ip() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectionButtonPressedTcp(
            "invalid_ip".to_string(),
            "8080".to_string(),
        ));
        assert_eq!(
            connect_dialog.tcp_connection_error,
            "Invalid IP Address: invalid IP address syntax"
        );
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_connection_button_pressed_tcp_invalid_port() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectionButtonPressedTcp(
            "192.168.1.1".to_string(),
            "invalid_port".to_string(),
        ));
        assert_eq!(
            connect_dialog.tcp_connection_error,
            "Invalid Port Number: invalid digit found in string"
        );
    }

    #[cfg(feature = "tcp")]
    #[test]
    fn test_connection_button_pressed_tcp_valid_ip_and_port() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(IpAddressEntered("192.168.1.1".to_string()));
        let _ = connect_dialog.update(PortNumberEntered("8080".to_string()));

        assert!(connect_dialog.tcp_connection_error.is_empty());
    }
}
