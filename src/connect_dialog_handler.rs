#![allow(unused)]

use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{self, column, container, row, text, text_input, Button, Text, Container};
use iced::{keyboard, Color, Command, Element, Event};
use iroh_net::relay::RelayUrl;
use iroh_net::NodeId;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ConnectDialog {
    pub node_id: String,
    pub relay_url: String,
    pub iroh_connection_error: String,
    pub show_modal: bool,
}
#[derive(Clone, Debug)]
pub enum ConnectDialogMessage {
    NodeId(String),
    RelayURL(String),
    ModalKeyEvent(Event),
    ConnectIroh(String, Option<String>),
    HideConnectDialog,
    ShowConnectDialog,
}

impl ConnectDialog {
    pub fn new() -> Self {
        Self {
            node_id: String::new(),
            relay_url: String::new(),
            iroh_connection_error: String::new(),
            show_modal: false,
        }
    }

    pub fn update(&mut self, message: ConnectDialogMessage) -> Command<Message> {
        match message {
            ConnectDialogMessage::ConnectIroh(node_id, relay_url) => {
                if node_id.trim().is_empty() {
                    self.iroh_connection_error = String::from("Pls Enter Node Id");
                    return Command::none();
                }
                if relay_url.clone().unwrap().trim().is_empty() {
                    self.iroh_connection_error = String::from("Pls Enter Relay Url");
                    return Command::none();
                }

                let node_id_result = NodeId::from_str(node_id.as_str().trim());
                match node_id_result {
                    Ok(_node_id) => {
                        let relay_url_result =
                            RelayUrl::from_str(relay_url.clone().unwrap().as_str().trim());
                        match relay_url_result {
                            Ok(_relay_url) => {
                                // TODO
                                // Make iroh connection
                                // Add spinner when establishing remote connection
                            }
                            Err(err) => {
                                self.iroh_connection_error = format!("{}", err);
                            }
                        }
                    }
                    Err(err) => {
                        self.iroh_connection_error = format!("{}", err);
                    }
                }

                return Command::none();
            }

            ConnectDialogMessage::ShowConnectDialog => {
                self.show_modal = true;
                return Command::none();
            }

            ConnectDialogMessage::HideConnectDialog => {
                self.hide_modal();
                return Command::none();
            }

            ConnectDialogMessage::ModalKeyEvent(event) => {
                return match event {
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
                };
            }

            ConnectDialogMessage::NodeId(node_id) => {
                self.node_id = node_id;
                return Command::none();
            }

            ConnectDialogMessage::RelayURL(relay_url) => {
                self.relay_url = relay_url;
                return Command::none();
            }
        }
    }

    fn hide_modal(&mut self) {
        self.show_modal = false;
        self.node_id.clear();
        self.relay_url.clear();
    }
}
