use self::SsidDialogMessage::{
    ConnectButtonPressedIroh, ConnectionError, HideSsidDialog, ModalKeyEvent, NodeIdEntered,
    RelayURL, ShowSsidDialog,
};

use crate::views::hardware_view::HardwareTarget::*;
use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{self, column, container, text, text_input, Button, Row, Text};
use iced::{keyboard, Element, Event, Length, Task};
use iced_futures::Subscription;
use iroh_net::{relay::RelayUrl, NodeId};
use std::str::FromStr;
use std::time::Duration;

use crate::hw_definition::description::{HardwareDescription, SsidSpec};
use crate::views::dialog_styles::{
    ACTIVE_TAB_BUTTON_STYLE, CONNECTION_ERROR_DISPLAY, INACTIVE_TAB_BUTTON_HOVER_STYLE,
    INACTIVE_TAB_BUTTON_STYLE, INFO_TEXT_STYLE, MODAL_CANCEL_BUTTON_HOVER_STYLE,
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_HOVER_STYLE, MODAL_CONNECT_BUTTON_STYLE,
    MODAL_CONTAINER_STYLE, TAB_BAR_STYLE, TEXT_BOX_CONTAINER_STYLE,
};
use iced::widget::button::Status::Hovered;
use std::sync::LazyLock;

const IROH_INFO_TEXT: &str = "To connect to a Pi using iroh-net, ensure piglet is running on the remote Pi. Retrieve the nodeid from piglet, enter it below, and optionally provide a Relay URL";

static IROH_INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub struct SsidDialog {
    nodeid: String,
    relay_url: String,
    iroh_connection_error: String,
    pub show_modal: bool,
    show_spinner: bool,
    disable_widgets: bool,
    display_iroh: bool,
}

#[derive(Clone, Debug)]
pub enum SsidDialogMessage {
    NodeIdEntered(String),
    RelayURL(String),
    ModalKeyEvent(Event),
    ConnectButtonPressedIroh(String, String),
    HideSsidDialog,
    ShowSsidDialog(HardwareDescription, Option<SsidSpec>),
    ConnectionError(String),
    PortNumberEntered(String),
}
impl Default for SsidDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SsidDialog {
    pub fn new() -> Self {
        Self {
            nodeid: String::new(),
            relay_url: String::new(),
            iroh_connection_error: String::new(),
            show_modal: false,
            show_spinner: false,
            disable_widgets: false,
            display_iroh: true,
        }
    }

    /// Set the error state of the dialog with a message to display
    pub fn set_error(&mut self, error: String) {
        self.iroh_connection_error.clone_from(&error);
    }

    async fn empty() {}

    pub fn update(&mut self, message: SsidDialogMessage) -> Task<Message> {
        match message {
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

            ShowSsidDialog(_hardware_description, _ssid_spec) => {
                self.show_modal = true;
                text_input::focus(IROH_INPUT_ID.clone())
            }

            HideSsidDialog => {
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

            NodeIdEntered(node_id) => {
                self.nodeid = node_id;
                Task::none()
            }

            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                Task::none()
            }

            ConnectionError(error) => {
                self.set_error(error);
                self.enable_widgets_and_hide_spinner();
                Task::none()
            }
            SsidDialogMessage::PortNumberEntered(_) => Task::none(),
        }
    }

    //noinspection RsLift
    pub fn view(&self) -> Element<'_, Message> {
        match self.disable_widgets {
            true => self.create_iroh_container(false),
            false => self.create_iroh_container(true),
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
    pub fn subscription(&self) -> Subscription<SsidDialogMessage> {
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

    fn create_connection_row_iroh(&self) -> Row<'_, Message> {
        if self.show_spinner && self.disable_widgets && self.display_iroh {
            self.create_spinner_row()
        } else {
            self.create_default_iroh_row()
        }
    }

    fn create_spinner_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .style(move |_theme, _status| ACTIVE_TAB_BUTTON_STYLE),
            )
            .push(
                Circular::new()
                    .easing(&EMPHASIZED_ACCELERATE)
                    .cycle_duration(Duration::from_secs_f32(2.0)),
            )
            .push(
                Button::new(Text::new("Connect")).style(move |_theme, status| {
                    if status == Hovered {
                        MODAL_CONNECT_BUTTON_HOVER_STYLE
                    } else {
                        MODAL_CONNECT_BUTTON_STYLE
                    }
                }),
            )
            .spacing(150)
            .align_y(iced::Alignment::Center)
    }

    fn create_default_iroh_row(&self) -> Row<'_, Message> {
        Row::new()
            .push(
                Button::new(Text::new("Cancel"))
                    .on_press(Message::SsidDialog(HideSsidDialog))
                    .style(|_, status| {
                        if status == Hovered {
                            MODAL_CANCEL_BUTTON_HOVER_STYLE
                        } else {
                            MODAL_CANCEL_BUTTON_STYLE
                        }
                    }),
            )
            .push(
                Button::new(Text::new("Connect"))
                    .on_press(Message::SsidDialog(ConnectButtonPressedIroh(
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
            .spacing(350)
            .align_y(iced::Alignment::Center)
    }

    fn create_text_container_iroh(&self) -> Element<'_, Message> {
        container(Text::new(IROH_INFO_TEXT).style(move |_theme| INFO_TEXT_STYLE))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| TEXT_BOX_CONTAINER_STYLE)
            .into()
    }

    //noinspection RsLiveness
    fn create_iroh_container(&self, input_enabled: bool) -> Element<'_, Message> {
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
                                .on_submit(Message::SsidDialog(ConnectButtonPressedIroh(
                                    self.nodeid.clone(),
                                    self.relay_url.clone(),
                                )));
                            if input_enabled {
                                node_input = node_input
                                    .on_input(|input| Message::SsidDialog(NodeIdEntered(input)));
                            }
                            node_input
                        }
                    ]
                    .spacing(10),
                    column![text("Relay URL (Optional)").size(12), {
                        let mut relay_input =
                            text_input("Enter Relay Url (Optional)", &self.relay_url)
                                .padding(5)
                                .on_submit(Message::SsidDialog(ConnectButtonPressedIroh(
                                    self.nodeid.clone(),
                                    self.relay_url.clone(),
                                )));
                        if input_enabled {
                            relay_input =
                                relay_input.on_input(|input| Message::SsidDialog(RelayURL(input)));
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

        container(button_row)
            .style(move |_theme| TAB_BAR_STYLE)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_show_connect_dialog() {
        let mut connect_dialog = SsidDialog::new();
        assert!(!connect_dialog.show_modal);

        let _ = connect_dialog.update(ShowSsidDialog);
        assert!(connect_dialog.show_modal);
    }

    #[test]
    fn test_hide_connect_dialog() {
        let mut connect_dialog = SsidDialog::new();
        connect_dialog.show_modal = true;

        let _ = connect_dialog.update(HideSsidDialog);
        assert!(!connect_dialog.show_modal);
        assert!(connect_dialog.nodeid.is_empty());
        assert!(connect_dialog.relay_url.is_empty());
        assert!(connect_dialog.iroh_connection_error.is_empty());
        assert!(!connect_dialog.show_spinner);
        assert!(!connect_dialog.disable_widgets);
    }

    #[test]
    fn test_node_id_entered() {
        let mut connect_dialog = SsidDialog::new();
        let node_id = "test_node_id".to_string();

        let _ = connect_dialog.update(NodeIdEntered(node_id.clone()));
        assert_eq!(connect_dialog.nodeid, node_id);
    }

    #[test]
    fn test_relay_url_entered() {
        let mut connect_dialog = SsidDialog::new();
        let relay_url = "test_relay_url".to_string();

        let _ = connect_dialog.update(RelayURL(relay_url.clone()));
        assert_eq!(connect_dialog.relay_url, relay_url);
    }

    #[test]
    fn test_connect_button_pressed_empty_node_id() {
        let mut connect_dialog = SsidDialog::new();
        let _ = connect_dialog.update(ConnectButtonPressedIroh("".to_string(), "".to_string()));
        assert_eq!(connect_dialog.iroh_connection_error, "Please Enter Node Id");
    }

    #[test]
    fn test_connect_button_pressed_invalid_node_id() {
        let mut connect_dialog = SsidDialog::new();
        let invalid_node_id = "invalid_node_id".to_string();

        let _ = connect_dialog.update(ConnectButtonPressedIroh(invalid_node_id, "".to_string()));
        assert!(!connect_dialog.iroh_connection_error.is_empty());
    }

    #[test]
    fn test_connection_error() {
        let mut connect_dialog = SsidDialog::new();
        let error_message = "Connection failed".to_string();

        let _ = connect_dialog.update(ConnectionError(error_message.clone()));
        assert_eq!(connect_dialog.iroh_connection_error, error_message);
    }
}
