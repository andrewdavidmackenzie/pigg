#![allow(unused)]

use crate::connect_dialog_handler::ConnectDialogMessage::{
    ConnectButtonPressed, ConnectionError, HideConnectDialog, ModalKeyEvent, NodeIdEntered,
    RelayURL, ShowConnectDialog,
};
use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
use crate::views::hardware_view::HardwareTarget::Remote;
use crate::views::message_row::MessageMessage;
use crate::views::message_row::MessageRowMessage::ShowStatusMessage;
use crate::Message::InfoRow;
use crate::{empty, Message};
use iced::keyboard::key;
use iced::widget::{
    self, column, container, row, text, text_input, tooltip, Button, Container, Row, Text,
};
use iced::{keyboard, Color, Command, Element, Event};
use iced_futures::Subscription;
use iroh_net::relay::RelayUrl;
use iroh_net::NodeId;
use std::str::FromStr;
use std::time::Duration;

use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;

#[derive(Debug, Clone)]
pub struct ConnectDialog {
    pub node_id: String,
    pub relay_url: String,
    pub iroh_connection_error: String,
    pub show_modal: bool,
    pub show_spinner: bool,
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
            node_id: String::new(),
            relay_url: String::new(),
            iroh_connection_error: String::new(),
            show_modal: false,
            show_spinner: false,
        }
    }

    pub fn update(&mut self, message: ConnectDialogMessage) -> Command<Message> {
        return match message {
            ConnectButtonPressed(node_id, mut url) => {
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
                                    self.iroh_connection_error = format!("{}", err);
                                    return Command::none();
                                }
                            }
                        };

                        return Command::perform(empty(), move |_| {
                            Message::ConnectRequest(Remote(nodeid, relay_url))
                        });
                    }
                    Err(err) => {
                        self.iroh_connection_error = format!("{}", err);
                        self.show_spinner = false;
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
                self.node_id = node_id;
                Command::none()
            }

            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                Command::none()
            }

            ConnectionError(error) => {
                self.iroh_connection_error = error;
                Command::none()
            }
        };
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        let modal_connect_button_style = ButtonStyle {
            bg_color: Color::new(0.0, 1.0, 1.0, 1.0), // Cyan background color
            text_color: Color::BLACK,
            hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0), // Darker cyan color when hovered
            hovered_text_color: Color::WHITE,
            border_radius: 2.0,
        };

        let modal_cancel_button_style = ButtonStyle {
            bg_color: Color::new(0.8, 0.0, 0.0, 1.0), // Gnome like Red background color
            text_color: Color::WHITE,
            hovered_bg_color: Color::new(0.9, 0.2, 0.2, 1.0), // Slightly lighter red when hovered
            hovered_text_color: Color::WHITE,
            border_radius: 2.0,
        };

        let text_box_container_style = ContainerStyle {
            border_color: Color::WHITE,
            background_color: Color::WHITE,
            border_width: 0.0,
            border_radius: 0.0,
        };

        let modal_container_style = ContainerStyle {
            border_color: Color::WHITE,
            background_color: Color::TRANSPARENT,
            border_radius: 2.0,
            border_width: 2.0,
        };

        let connection_error_display = TextStyle {
            text_color: Color::new(1.0, 0.0, 0.0, 0.5),
        };

        let iroh_info_text = TextStyle {
            text_color: Color::BLACK,
        };

        let mut connection_row = Row::new();

        let spinner_connection_row = Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(
                        ConnectDialogMessage::HideConnectDialog,
                    ))
                    .style(modal_cancel_button_style.get_button_style()),
            )
            .push(
                Circular::new()
                    .easing(&EMPHASIZED_ACCELERATE)
                    .cycle_duration(Duration::from_secs_f32(2.0)),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(
                        ConnectDialogMessage::ConnectButtonPressed(
                            self.node_id.clone(),
                            self.relay_url.clone(),
                        ),
                    ))
                    .style(modal_connect_button_style.get_button_style()),
            )
            .spacing(160)
            .align_items(iced::Alignment::Center);

        let without_spinner_connection_row = Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::ConnectDialog(
                        ConnectDialogMessage::HideConnectDialog,
                    ))
                    .style(modal_cancel_button_style.get_button_style()),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::ConnectDialog(
                        ConnectDialogMessage::ConnectButtonPressed(
                            self.node_id.clone(),
                            self.relay_url.clone(),
                        ),
                    ))
                    .style(modal_connect_button_style.get_button_style()),
            )
            .spacing(360)
            .align_items(iced::Alignment::Center);

        if self.show_spinner {
            connection_row = spinner_connection_row;
        } else {
            connection_row = without_spinner_connection_row;
        }

        let text_container =  container(
            Text::new("To connect to a remote node using iroh-net, ensure piglet is running on the remote node. Retrieve the node id from piglet, enter it below, and optionally provide a Relay URL.").style(iroh_info_text.get_text_color())
        ).padding(5).style(text_box_container_style.get_container_style());

        container(
            column![column![
                text("Connect To Remote Pi").size(20),
                column![
                    text_container,
                    text(self.iroh_connection_error.clone())
                        .style(connection_error_display.get_text_color()),
                    text("Node Id").size(12),
                    text_input("Enter node id", &self.node_id)
                        .on_input(|input| Message::ConnectDialog(
                            ConnectDialogMessage::NodeIdEntered(input)
                        ))
                        .padding(5),
                ]
                .spacing(10),
                column![
                    text("Relay URL (Optonal) ").size(12),
                    text_input("Enter Relay Url (Optional)", &self.relay_url)
                        .on_input(
                            |input| Message::ConnectDialog(ConnectDialogMessage::RelayURL(input))
                        )
                        .padding(5),
                ]
                .spacing(5),
                connection_row,
            ]
            .spacing(10)]
            .spacing(20),
        )
        .style(modal_container_style.get_container_style())
        .width(520)
        .padding(15)
        .into()
    }

    fn hide_modal(&mut self) {
        self.show_modal = false; // Hide the dialog
        self.node_id.clear(); // Clear the node id, on Cancel
        self.iroh_connection_error.clear(); // Clear the error, on Cancel
        self.relay_url.clear(); // Clear the relay url, on Cancel
        self.show_spinner = false; // Hide spinner, on Cancel
    }

    // Handle Keyboard events
    pub fn subscription(&self) -> Subscription<ConnectDialogMessage> {
        iced::event::listen().map(ModalKeyEvent)
    }
}
