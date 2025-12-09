use self::SsidDialogMessage::{
    ConnectionError, HideSsidDialog, ModalKeyEvent, NameEntered, PasswordEntered,
    SendButtonPressed, Show,
};

use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{
    self, checkbox, column, container, pick_list, row, text, text_input, Button, Row, Text,
};
use iced::{keyboard, Element, Event, Length, Task};
use iced_futures::Subscription;
use std::time::Duration;

use crate::views::dialog_styles::{
    cancel_button, connect_button, CONNECTION_ERROR_DISPLAY, INFO_TEXT_STYLE,
    MODAL_CONTAINER_STYLE, TEXT_BOX_CONTAINER_STYLE,
};
use crate::views::ssid_dialog::SsidDialogMessage::{HidePasswordToggled, SecuritySelected};
use pigdef::description::HardwareDetails;
use pigdef::description::SsidSpec;
use pignet::usb_host;
use std::sync::LazyLock;

static INPUT_ID: LazyLock<widget::Id> = LazyLock::new(widget::Id::unique);

#[derive(Debug, Clone)]
pub struct SsidDialog {
    ssid_spec: SsidSpec,
    error: String,
    pub show_modal: bool,
    show_spinner: bool,
    disable_widgets: bool,
    hardware_details: HardwareDetails,
    hide_password: bool,
}

#[derive(Clone, Debug)]
pub enum SsidDialogMessage {
    NameEntered(String),
    PasswordEntered(String),
    SecuritySelected(String),
    ModalKeyEvent(Event),
    SendButtonPressed(String, String, String),
    HideSsidDialog,
    /// Show the dialog to configure Wi-Fi details of a connected device
    Show(HardwareDetails, Option<SsidSpec>),
    ConnectionError(String),
    HidePasswordToggled,
}

#[allow(unused_variables)]
fn send_ssid(serial_number: String, ssid_spec: SsidSpec) -> Task<Message> {
    #[cfg(feature = "usb")]
    return Task::perform(usb_host::send_ssid_spec(serial_number, ssid_spec), |res| {
        Message::SsidSpecSent(res.map_err(|e| format!("Could not send SSID Spec: {e}")))
    });
    #[cfg(not(feature = "usb"))]
    Task::none()
}

impl SsidDialog {
    pub fn new() -> Self {
        Self {
            ssid_spec: SsidSpec::default(),
            error: String::new(),
            show_modal: false,
            show_spinner: false,
            disable_widgets: false,
            hardware_details: HardwareDetails::default(),
            hide_password: true,
        }
    }

    fn modal_key_event(&mut self, event: Event) -> Task<Message> {
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

    pub fn update(&mut self, message: SsidDialogMessage) -> Task<Message> {
        match message {
            SendButtonPressed(name, password, security) => {
                match SsidSpec::try_new(name, password, security) {
                    Ok(ssid_spec) => {
                        self.error.clear();
                        self.show_spinner = true;
                        self.disable_widgets = true;
                        send_ssid(self.hardware_details.serial.clone(), ssid_spec)
                    }
                    Err(error) => {
                        self.set_error(error);
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Task::none()
                    }
                }
            }

            Show(hardware_details, wifi) => {
                self.hardware_details = hardware_details;
                self.ssid_spec = wifi.unwrap_or(SsidSpec::default());
                self.show_modal = true;
                text_input::focus(INPUT_ID.clone())
            }

            HideSsidDialog => {
                self.hide_modal();
                Task::none()
            }

            ModalKeyEvent(event) => self.modal_key_event(event),

            NameEntered(name) => {
                self.ssid_spec.ssid_name = name;
                Task::none()
            }

            PasswordEntered(password) => {
                self.ssid_spec.ssid_pass = password;
                Task::none()
            }

            SecuritySelected(security) => {
                self.ssid_spec.ssid_security = security.to_string();
                Task::none()
            }

            ConnectionError(error) => {
                self.error = error;
                self.disable_widgets = false;
                self.show_spinner = false;
                Task::none()
            }

            HidePasswordToggled => {
                self.hide_password = !self.hide_password;
                Task::none()
            }
        }
    }

    //noinspection RsLift
    pub fn view(&self) -> Element<'_, Message> {
        let security_options = vec![
            "open".to_string(),
            "wpa".to_string(),
            "wpa2".to_string(),
            "wpa3".to_string(),
        ];

        container(
            column![
                text("Configure Wi-Fi of USB connected 'porky' device").size(20),
                self.create_text_container(format!("To configure the Wi-Fi of the USB connected device '{}' with Serial Number '{}' \
                 complete the fields below and then press 'Send'",
                    self.hardware_details.model, self.hardware_details.serial
                )),
                text(self.error.clone()).style(move |_theme| { CONNECTION_ERROR_DISPLAY }),
                text("SSID Name"),
                {
                    let mut name_input = text_input("Enter SSID Name", &self.ssid_spec.ssid_name)
                        .padding(5)
                        .id(INPUT_ID.clone())
                        .on_submit(Message::SsidDialog(SendButtonPressed(
                            self.ssid_spec.ssid_name.clone(),
                            self.ssid_spec.ssid_pass.clone(),
                            self.ssid_spec.ssid_security.clone(),
                        )));
                    if !self.disable_widgets {
                        name_input =
                            name_input.on_input(|input| Message::SsidDialog(NameEntered(input)));
                    }
                    name_input
                },
                row![
                    text("SSID Password"),
                    horizontal_space(),
                    checkbox("Hide Password", self.hide_password)
                        .on_toggle(|_| Message::SsidDialog(HidePasswordToggled))
                ],
                {
                    let mut password_input =
                        text_input("Enter SSID Password", &self.ssid_spec.ssid_pass)
                            .secure(self.hide_password)
                            .padding(5)
                            .on_submit(Message::SsidDialog(SendButtonPressed(
                                self.ssid_spec.ssid_name.clone(),
                                self.ssid_spec.ssid_pass.clone(),
                                self.ssid_spec.ssid_security.clone(),
                            )));
                    if !self.disable_widgets {
                        password_input = password_input
                            .on_input(|input| Message::SsidDialog(PasswordEntered(input)));
                    }
                    password_input
                },
                text("SSID Security"),
                pick_list(
                    security_options,
                    Some(self.ssid_spec.ssid_security.clone()),
                    move |selected| { Message::SsidDialog(SecuritySelected(selected)) }
                )
                .padding(5)
                .placeholder("Select SSID Security"),
                self.send_row(),
            ]
            .spacing(10),
        )
        .style(move |_theme| MODAL_CONTAINER_STYLE)
        .width(520)
        .padding(15)
        .into()
    }

    pub fn hide_modal(&mut self) {
        self.show_modal = false; // Hide the dialog
        self.error.clear(); // Clear the error, on Cancel
        self.show_spinner = false; // Hide spinner, on Cancel
        self.disable_widgets = false; // Enable widgets, on Cancel
    }

    // Handle Keyboard events
    pub fn subscription(&self) -> Subscription<SsidDialogMessage> {
        iced::event::listen().map(ModalKeyEvent)
    }

    pub fn enable_widgets_and_hide_spinner(&mut self) {
        self.disable_widgets = false;
        self.show_spinner = false;
    }
    pub fn set_error(&mut self, error: String) {
        self.error = error;
    }

    fn send_row(&self) -> Row<'_, Message> {
        let mut cancel_button = Button::new(Text::new("Cancel")).style(cancel_button);

        let mut send_button = Button::new(Text::new("Send")).style(connect_button);

        let mut row = Row::new().align_y(iced::Alignment::Center);

        if self.show_spinner {
            row = row
                .push(cancel_button)
                .push(horizontal_space())
                .push(
                    Circular::new()
                        .easing(&EMPHASIZED_ACCELERATE)
                        .cycle_duration(Duration::from_secs_f32(2.0)),
                )
                .push(horizontal_space())
        } else {
            cancel_button = cancel_button.on_press(Message::SsidDialog(HideSsidDialog));
            send_button = send_button.on_press(Message::SsidDialog(SendButtonPressed(
                self.ssid_spec.ssid_name.clone(),
                self.ssid_spec.ssid_pass.clone(),
                self.ssid_spec.ssid_security.clone(),
            )));
            row = row.push(cancel_button).push(horizontal_space());
        }

        row.push(send_button)
    }

    fn create_text_container(&self, msg: String) -> Element<'_, Message> {
        container(Text::new(msg).style(move |_theme| INFO_TEXT_STYLE))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| TEXT_BOX_CONTAINER_STYLE)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_show_ssid_dialog() {
        let mut ssid_dialog = SsidDialog::new();
        assert!(!ssid_dialog.show_modal);

        let _ = ssid_dialog.update(Show(HardwareDetails::default(), Some(SsidSpec::default())));
        assert!(ssid_dialog.show_modal);
    }

    #[test]
    fn test_show_empty_ssid_dialog() {
        let mut ssid_dialog = SsidDialog::new();
        assert!(!ssid_dialog.show_modal);

        let _ = ssid_dialog.update(Show(HardwareDetails::default(), None));
        assert!(ssid_dialog.show_modal);
    }

    #[test]
    fn test_hide_connect_dialog() {
        let mut ssid_dialog = SsidDialog::new();
        ssid_dialog.show_modal = true;

        let _ = ssid_dialog.update(HideSsidDialog);
        assert!(!ssid_dialog.show_modal);
        assert!(ssid_dialog.ssid_spec.ssid_name.is_empty());
        assert!(ssid_dialog.ssid_spec.ssid_pass.is_empty());
        assert!(ssid_dialog.error.is_empty());
        assert!(!ssid_dialog.show_spinner);
        assert!(!ssid_dialog.disable_widgets);
    }

    #[test]
    fn test_name_entered() {
        let mut ssid_dialog = SsidDialog::new();
        let name = "test_name".to_string();
        let _ = ssid_dialog.update(NameEntered(name.clone()));
        assert_eq!(ssid_dialog.ssid_spec.ssid_name, name);
    }

    #[test]
    fn test_password_entered() {
        let mut ssid_dialog = SsidDialog::new();
        let password = "test_password".to_string();
        let _ = ssid_dialog.update(PasswordEntered(password.clone()));
        assert_eq!(ssid_dialog.ssid_spec.ssid_pass, password);
    }

    #[test]
    fn test_send_button_pressed_empty_name() {
        let mut ssid_dialog = SsidDialog::new();
        let _ = ssid_dialog.update(SendButtonPressed(
            "".to_string(),
            "".to_string(),
            "open".to_string(),
        ));
        assert_eq!(ssid_dialog.error, "Please Enter SSID name");
    }

    #[test]
    fn test_send_button_pressed_invalid_name() {
        let mut ssid_dialog = SsidDialog::new();
        let _ = ssid_dialog.update(SendButtonPressed(
            "".to_string(),
            "".to_string(),
            "open".to_string(),
        ));
        assert!(!ssid_dialog.error.is_empty());
    }

    #[test]
    fn test_send_error() {
        let mut ssid_dialog = SsidDialog::new();
        let error_message = "Connection failed".to_string();

        let _ = ssid_dialog.update(ConnectionError(error_message.clone()));
        assert_eq!(ssid_dialog.error, error_message);
    }
}
