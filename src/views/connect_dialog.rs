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
#[cfg(feature = "tcp")]
use crate::HardwareTarget::Tcp;
#[cfg(feature = "tcp")]
use std::net::IpAddr;

#[cfg(feature = "iroh")]
use crate::views::hardware_view::HardwareTarget::*;
use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;
use crate::Message;
use iced::keyboard::key;
#[allow(unused_imports)]
use iced::widget::{self, column, container, text, text_input, Button, Row, Text};
use iced::{keyboard, Element, Event, Length, Task};
use iced_futures::Subscription;
#[cfg(feature = "iroh")]
use iroh_net::{relay::RelayUrl, NodeId};
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "iroh")]
const IROH_INFO_TEXT: &str = "To connect to a Pi using iroh-net, ensure piglet is running on the remote Pi. Retrieve the nodeid from piglet, enter it below, and optionally provide a Relay URL";
#[cfg(feature = "tcp")]
const TCP_INFO_TEXT: &str = "To connect to a Pi/Pi Pico using TCP, ensure it is reachable over the network. Retrieve the device's IP address and the port number from it (see piglet or porky docs) and enter below.";

use crate::views::dialog_styles::{
    ACTIVE_TAB_BUTTON_STYLE, CONNECTION_ERROR_DISPLAY, INACTIVE_TAB_BUTTON_HOVER_STYLE,
    INACTIVE_TAB_BUTTON_STYLE, INFO_TEXT_STYLE, MODAL_CANCEL_BUTTON_HOVER_STYLE,
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_HOVER_STYLE, MODAL_CONNECT_BUTTON_STYLE,
    MODAL_CONTAINER_STYLE, TAB_BAR_STYLE, TEXT_BOX_CONTAINER_STYLE,
};
use iced::widget::button::Status::Hovered;
use iced::widget::horizontal_space;
use std::sync::LazyLock;

#[cfg(feature = "tcp")]
static TCP_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);
static IROH_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

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
            self.iroh_connection_error.clone_from(&error);
        }
        #[cfg(feature = "tcp")]
        {
            self.tcp_connection_error.clone_from(&error);
        }
    }

    async fn empty() {}

    pub fn update(&mut self, message: ConnectDialogMessage) -> Task<Message> {
        match message {
            #[cfg(feature = "tcp")]
            ConnectionButtonPressedTcp(ip_address, port_num) => {
                // Display error when Ip address field is empty
                if ip_address.trim().is_empty() {
                    self.tcp_connection_error = String::from("Please Enter IP Address");
                    return Task::none();
                }

                // Display error when port number field is empty
                if port_num.trim().is_empty() {
                    self.tcp_connection_error = String::from("Please Enter Port Number");
                    return Task::none();
                }

                // Validate IP address
                match IpAddr::from_str(ip_address.as_str().trim()) {
                    Ok(ip) => {
                        // Validate port number
                        match port_num.trim().parse::<u16>() {
                            Ok(port) => {
                                self.tcp_connection_error.clear();

                                // Proceed to request connection when the port number is valid
                                Task::perform(Self::empty(), move |_| {
                                    Message::ConnectRequest(Tcp(ip, port))
                                })
                            }
                            Err(e) => {
                                self.tcp_connection_error = format!("Invalid Port Number: {}", e);
                                self.show_spinner = false;
                                self.disable_widgets = false;
                                Task::none()
                            }
                        }
                    }
                    Err(err) => {
                        self.tcp_connection_error = format!("Invalid IP Address: {}", err);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Task::none()
                    }
                }
            }

            #[cfg(feature = "iroh")]
            ConnectButtonPressedIroh(node_id, url) => {
                if node_id.trim().is_empty() {
                    self.iroh_connection_error = String::from("Please Enter Node Id");
                    return Task::none();
                }

                match NodeId::from_str(node_id.as_str().trim()) {
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
                                    return Task::none();
                                }
                            }
                        };

                        Task::perform(Self::empty(), move |_| {
                            Message::ConnectRequest(Iroh(nodeid, relay_url.clone()))
                        })
                    }
                    Err(err) => {
                        self.iroh_connection_error = format!("{}", err);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Task::none()
                    }
                }
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
                Task::none()
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
                        Task::none()
                    }

                    _ => Task::none(),
                }
            }

            #[cfg(feature = "tcp")]
            IpAddressEntered(ip_addr) => {
                self.ip_address = ip_addr;
                Task::none()
            }

            #[cfg(feature = "tcp")]
            PortNumberEntered(port_num) => {
                self.port_number = port_num;
                Task::none()
            }

            #[cfg(feature = "iroh")]
            NodeIdEntered(node_id) => {
                self.nodeid = node_id;
                Task::none()
            }

            #[cfg(feature = "iroh")]
            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                Task::none()
            }

            ConnectionError(error) => {
                self.set_error(error);
                self.enable_widgets_and_hide_spinner();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.display_iroh {
            #[cfg(feature = "iroh")]
            return self.create_iroh_container();
            #[cfg(not(feature = "iroh"))]
            return self.create_tcp_container();
        } else {
            #[cfg(feature = "tcp")]
            return self.create_tcp_container();
            #[cfg(not(feature = "tcp"))]
            return self.create_iroh_container();
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
                    .style(move |_theme, _status| ACTIVE_TAB_BUTTON_STYLE),
            )
            .push(horizontal_space())
            .push(
                Circular::new()
                    .easing(&EMPHASIZED_ACCELERATE)
                    .cycle_duration(Duration::from_secs_f32(2.0)),
            )
            .push(horizontal_space())
            .push(
                Button::new(Text::new("Connect")).style(move |_theme, status| {
                    if status == Hovered {
                        MODAL_CONNECT_BUTTON_HOVER_STYLE
                    } else {
                        MODAL_CONNECT_BUTTON_STYLE
                    }
                }),
            )
            .align_y(iced::Alignment::Center)
    }

    #[cfg(feature = "iroh")]
    fn create_default_iroh_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(HideConnectDialog))
                    .style(|_, status| {
                        if status == Hovered {
                            MODAL_CANCEL_BUTTON_HOVER_STYLE
                        } else {
                            MODAL_CANCEL_BUTTON_STYLE
                        }
                    }),
            )
            .push(horizontal_space())
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(ConnectButtonPressedIroh(
                        self.nodeid.clone(),
                        self.relay_url.clone(),
                    )))
                    .style(move |_theme, status| {
                        if status == Hovered {
                            MODAL_CONNECT_BUTTON_HOVER_STYLE
                        } else {
                            MODAL_CONNECT_BUTTON_STYLE
                        }
                    }),
            )
            .align_y(iced::Alignment::Center)
    }

    #[cfg(feature = "tcp")]
    fn create_default_tcp_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(HideConnectDialog))
                    .style(|_, status| {
                        if status == Hovered {
                            MODAL_CANCEL_BUTTON_HOVER_STYLE
                        } else {
                            MODAL_CANCEL_BUTTON_STYLE
                        }
                    }),
            )
            .push(horizontal_space())
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(ConnectionButtonPressedTcp(
                        self.ip_address.clone(),
                        self.port_number.clone(),
                    )))
                    .style(move |_theme, status| {
                        if status == Hovered {
                            MODAL_CONNECT_BUTTON_HOVER_STYLE
                        } else {
                            MODAL_CONNECT_BUTTON_STYLE
                        }
                    }),
            )
            .align_y(iced::Alignment::Center)
    }

    #[cfg(feature = "iroh")]
    fn create_text_container_iroh(&self) -> Element<'_, Message> {
        container(Text::new(IROH_INFO_TEXT).style(move |_theme| INFO_TEXT_STYLE))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| TEXT_BOX_CONTAINER_STYLE)
            .into()
    }

    #[cfg(feature = "tcp")]
    fn create_tcp_text_container(&self) -> Element<'_, Message> {
        container(Text::new(TCP_INFO_TEXT).style(move |_theme| INFO_TEXT_STYLE))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| TEXT_BOX_CONTAINER_STYLE)
            .into()
    }

    //noinspection RsLiveness
    #[cfg(feature = "iroh")]
    fn create_iroh_container(&self) -> Element<'_, Message> {
        container(
            column![
                self.create_tab_buttons(true),
                column![
                    text("Connect To Remote Pi Using Iroh").size(20),
                    column![
                        self.create_text_container_iroh(),
                        text(self.iroh_connection_error.clone())
                            .style(move |_theme| { CONNECTION_ERROR_DISPLAY }),
                        text("Node Id").size(12),
                        {
                            let mut node_input = text_input("Enter node id", &self.nodeid)
                                .padding(5)
                                .id(IROH_INPUT_ID.clone())
                                .on_submit(Message::ConnectDialog(ConnectButtonPressedIroh(
                                    self.nodeid.clone(),
                                    self.relay_url.clone(),
                                )));
                            if !self.disable_widgets {
                                node_input = node_input
                                    .on_input(|input| Message::ConnectDialog(NodeIdEntered(input)));
                            }
                            node_input
                        }
                    ]
                    .spacing(10),
                    column![text("Relay URL (Optional)").size(12), {
                        let mut relay_input =
                            text_input("Enter Relay Url (Optional)", &self.relay_url)
                                .padding(5)
                                .on_submit(Message::ConnectDialog(ConnectButtonPressedIroh(
                                    self.nodeid.clone(),
                                    self.relay_url.clone(),
                                )));
                        if !self.disable_widgets {
                            relay_input = relay_input
                                .on_input(|input| Message::ConnectDialog(RelayURL(input)));
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
        .style(move |_theme| MODAL_CONTAINER_STYLE)
        .width(520)
        .padding(15)
        .into()
    }

    //noinspection RsLiveness
    #[cfg(feature = "tcp")]
    fn create_tcp_container(&self) -> Element<'_, Message> {
        container(
            column![
                self.create_tab_buttons(false),
                column![
                    text("Connect To Remote Pi Using Tcp").size(20),
                    column![
                        self.create_tcp_text_container(),
                        text(self.tcp_connection_error.clone())
                            .style(move |_theme| { CONNECTION_ERROR_DISPLAY }),
                        text("IP Address").size(12),
                        {
                            let mut ip_input = text_input("Enter IP Address", &self.ip_address)
                                .padding(5)
                                .id(TCP_INPUT_ID.clone())
                                .on_submit(Message::ConnectDialog(ConnectionButtonPressedTcp(
                                    self.ip_address.clone(),
                                    self.port_number.clone(),
                                )));
                            if !self.disable_widgets {
                                ip_input = ip_input.on_input(|input| {
                                    Message::ConnectDialog(IpAddressEntered(input))
                                });
                            }
                            ip_input
                        }
                    ]
                    .spacing(10),
                    column![text("Port Number").size(12), {
                        let mut port_input = text_input("Enter Port Number", &self.port_number)
                            .padding(5)
                            .on_submit(Message::ConnectDialog(ConnectionButtonPressedTcp(
                                self.ip_address.clone(),
                                self.port_number.clone(),
                            )));
                        if !self.disable_widgets {
                            port_input = port_input
                                .on_input(|input| Message::ConnectDialog(PortNumberEntered(input)));
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
        .style(|_theme| MODAL_CONTAINER_STYLE)
        .width(520)
        .padding(15)
        .into()
    }

    fn create_tab_buttons(&self, is_iroh_active: bool) -> Element<'_, Message> {
        #[allow(unused_variables)]
        let (iroh_styles, tcp_styles) = if is_iroh_active {
            (
                (ACTIVE_TAB_BUTTON_STYLE, ACTIVE_TAB_BUTTON_STYLE),
                (INACTIVE_TAB_BUTTON_STYLE, INACTIVE_TAB_BUTTON_HOVER_STYLE),
            )
        } else {
            (
                (INACTIVE_TAB_BUTTON_STYLE, INACTIVE_TAB_BUTTON_HOVER_STYLE),
                (ACTIVE_TAB_BUTTON_STYLE, ACTIVE_TAB_BUTTON_STYLE),
            )
        };

        let button_row = Row::new().spacing(5);

        #[cfg(feature = "iroh")]
        let button_row = button_row.push(
            Button::new(Text::new("Connect using Iroh").width(Length::Fill).size(20))
                .on_press(Message::ConnectDialog(DisplayIrohTab))
                .style(move |_theme, status| {
                    if status == Hovered {
                        iroh_styles.1
                    } else {
                        iroh_styles.0
                    }
                })
                .width(260),
        );

        #[cfg(feature = "tcp")]
        let button_row = button_row.push(
            Button::new(Text::new("Connect using TCP").width(Length::Fill).size(20))
                .on_press(Message::ConnectDialog(DisplayTcpTab))
                .style(move |_theme, status| {
                    if status == Hovered {
                        tcp_styles.1
                    } else {
                        tcp_styles.0
                    }
                })
                .width(280),
        );

        container(button_row)
            .style(move |_theme| TAB_BAR_STYLE)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "tcp")]
    use crate::views::connect_dialog::ConnectDialogMessage::ConnectionButtonPressedTcp;
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
