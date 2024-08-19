use crate::connect_dialog_handler::ConnectDialogMessage::{
    ConnectButtonPressed, ConnectionError, HideConnectDialog, ModalKeyEvent, NodeIdEntered,
    RelayURL, ShowConnectDialog,
};
use crate::modal_handler::{
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_STYLE, MODAL_CONTAINER_STYLE,
};

use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
#[cfg(feature = "iroh")]
use crate::views::hardware_view::HardwareTarget::Iroh;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{self, column, container, text, text_input, Button, Row, Text};
use iced::{keyboard, Color, Command, Element, Event};
use iced_futures::Subscription;
use iroh_net::relay::RelayUrl;
use iroh_net::NodeId;
use std::str::FromStr;
use std::time::Duration;

use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;

const IROH_INFO_TEXT: &str = "To connect to a remote Pi using iroh-net, ensure piglet is running on the remote Pi. Retrieve the nodeid from piglet, enter it below, and optionally provide a Relay URL";

const IROH_INFO_TEXT_STYLE: TextStyle = TextStyle {
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

#[derive(Debug, Clone)]
pub struct ConnectDialog {
    nodeid: String,
    relay_url: String,
    iroh_connection_error: String,
    pub show_modal: bool,
    show_spinner: bool,
    disable_widgets: bool,
}

#[derive(Clone, Debug)]
pub enum ConnectDialogMessage {
    NodeIdEntered(String),
    RelayURL(String),
    ModalKeyEvent(Event),
    ConnectButtonPressed(String, String),
    HideConnectDialog,
    ShowConnectDialog,
    ConnectionError(String),
}
impl Default for ConnectDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectDialog {
    pub fn new() -> Self {
        Self {
            nodeid: String::new(),
            relay_url: String::new(),
            iroh_connection_error: String::new(),
            show_modal: false,
            show_spinner: false,
            disable_widgets: false,
        }
    }

    /// Set the error state of the dialog with a message to display
    pub fn set_error(&mut self, error: String) {
        self.iroh_connection_error = error;
    }

    async fn empty() {}

    pub fn update(&mut self, message: ConnectDialogMessage) -> Command<Message> {
        return match message {
            ConnectButtonPressed(node_id, url) => {
                if node_id.trim().is_empty() {
                    self.iroh_connection_error = String::from("Please Enter Node Id");
                    return Command::none();
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
                                    return Command::none();
                                }
                            }
                        };

                        return Command::perform(Self::empty(), move |_| {
                            Message::ConnectRequest(Iroh(nodeid, relay_url))
                        });
                    }
                    Err(err) => {
                        self.iroh_connection_error = format!("{}", err);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Command::none()
                    }
                }
            }

            ShowConnectDialog => {
                self.show_modal = true;
                Command::none()
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

            NodeIdEntered(node_id) => {
                self.nodeid = node_id;
                Command::none()
            }

            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                Command::none()
            }

            ConnectionError(error) => {
                self.set_error(error);
                self.enable_widgets_and_hide_spinner();
                Command::none()
            }
        };
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        let connection_row = if self.show_spinner && self.disable_widgets {
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
        } else {
            Row::new()
                .push(
                    Button::new(Text::new("Cancel"))
                        .on_press(Message::ConnectDialog(
                            ConnectDialogMessage::HideConnectDialog,
                        ))
                        .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                )
                .push(
                    Button::new(Text::new("Connect"))
                        .on_press(Message::ConnectDialog(
                            ConnectDialogMessage::ConnectButtonPressed(
                                self.nodeid.clone(),
                                self.relay_url.clone(),
                            ),
                        ))
                        .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                )
                .spacing(360)
                .align_items(iced::Alignment::Center)
        };

        let text_container =
            container(Text::new(IROH_INFO_TEXT).style(IROH_INFO_TEXT_STYLE.get_text_color()))
                .padding(10)
                .style(TEXT_BOX_CONTAINER_STYLE.get_container_style());

        if self.disable_widgets {
            container(
                column![column![
                    text("Connect To Remote Pi").size(20),
                    column![
                        text_container,
                        text(self.iroh_connection_error.clone())
                            .style(CONNECTION_ERROR_DISPLAY.get_text_color()),
                        text("Node Id").size(12),
                        text_input("Enter node id", &self.nodeid).padding(5),
                    ]
                    .spacing(10),
                    column![
                        text("Relay URL (Optional) ").size(12),
                        text_input("Enter Relay Url (Optional)", &self.relay_url).padding(5),
                    ]
                    .spacing(5),
                    connection_row,
                ]
                .spacing(10)]
                .spacing(20),
            )
            .style(MODAL_CONTAINER_STYLE.get_container_style())
            .width(520)
            .padding(15)
            .into()
        } else {
            container(
                column![column![
                    text("Connect To Remote Pi").size(20),
                    column![
                        text_container,
                        text(self.iroh_connection_error.clone())
                            .style(CONNECTION_ERROR_DISPLAY.get_text_color()),
                        text("Node Id").size(12),
                        text_input("Enter node id", &self.nodeid)
                            .on_input(|input| Message::ConnectDialog(
                                ConnectDialogMessage::NodeIdEntered(input)
                            ))
                            .padding(5),
                    ]
                    .spacing(10),
                    column![
                        text("Relay URL (Optional) ").size(12),
                        text_input("Enter Relay Url (Optional)", &self.relay_url)
                            .on_input(|input| Message::ConnectDialog(
                                ConnectDialogMessage::RelayURL(input)
                            ))
                            .padding(5),
                    ]
                    .spacing(5),
                    connection_row,
                ]
                .spacing(10)]
                .spacing(20),
            )
            .style(MODAL_CONTAINER_STYLE.get_container_style())
            .width(520)
            .padding(15)
            .into()
        }
    }

    pub fn hide_modal(&mut self) {
        self.show_modal = false; // Hide the dialog
        self.nodeid.clear(); // Clear the node id, on Cancel
        self.iroh_connection_error.clear(); // Clear the error, on Cancel
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_show_connect_dialog() {
        let mut connect_dialog = ConnectDialog::new();
        assert!(!connect_dialog.show_modal);

        let _ = connect_dialog.update(ShowConnectDialog);
        assert!(connect_dialog.show_modal);
    }

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

    #[test]
    fn test_node_id_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let node_id = "test_node_id".to_string();

        let _ = connect_dialog.update(NodeIdEntered(node_id.clone()));
        assert_eq!(connect_dialog.nodeid, node_id);
    }

    #[test]
    fn test_relay_url_entered() {
        let mut connect_dialog = ConnectDialog::new();
        let relay_url = "test_relay_url".to_string();

        let _ = connect_dialog.update(RelayURL(relay_url.clone()));
        assert_eq!(connect_dialog.relay_url, relay_url);
    }

    #[test]
    fn test_connect_button_pressed_empty_node_id() {
        let mut connect_dialog = ConnectDialog::new();
        let _ = connect_dialog.update(ConnectButtonPressed("".to_string(), "".to_string()));
        assert_eq!(connect_dialog.iroh_connection_error, "Please Enter Node Id");
    }

    #[test]
    fn test_connect_button_pressed_invalid_node_id() {
        let mut connect_dialog = ConnectDialog::new();
        let invalid_node_id = "invalid_node_id".to_string();

        let _ = connect_dialog.update(ConnectButtonPressed(invalid_node_id, "".to_string()));
        assert!(!connect_dialog.iroh_connection_error.is_empty());
    }

    #[test]
    fn test_connection_error() {
        let mut connect_dialog = ConnectDialog::new();
        let error_message = "Connection failed".to_string();

        let _ = connect_dialog.update(ConnectionError(error_message.clone()));
        assert_eq!(connect_dialog.iroh_connection_error, error_message);
    }
}
