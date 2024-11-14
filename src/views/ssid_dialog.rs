use self::SsidDialogMessage::{
    ConnectionError, HideSsidDialog, ModalKeyEvent, NameEntered, PasswordEntered,
    SendButtonPressed, ShowSsidDialog,
};
use std::fmt::{Display, Formatter};

use crate::widgets::spinner::circular::Circular;
use crate::widgets::spinner::easing::EMPHASIZED_ACCELERATE;
use crate::{usb_raw, Message};
use iced::keyboard::key;
use iced::widget::{self, column, container, text, text_input, Button, Row, Text};
use iced::{keyboard, Element, Event, Length, Task};
use iced_futures::Subscription;
use std::time::Duration;

use crate::hw_definition::description::{
    HardwareDetails, SsidSpec, SSID_NAME_LENGTH, SSID_PASS_LENGTH,
};
use crate::views::dialog_styles::{
    CONNECTION_ERROR_DISPLAY, INFO_TEXT_STYLE, MODAL_CANCEL_BUTTON_HOVER_STYLE,
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_HOVER_STYLE, MODAL_CONNECT_BUTTON_STYLE,
    MODAL_CONTAINER_STYLE, TEXT_BOX_CONTAINER_STYLE,
};
use crate::views::message_box::MessageMessage::{Error, Info};
#[cfg(feature = "usb-raw")]
use crate::views::message_box::MessageRowMessage::ShowStatusMessage;
use crate::views::ssid_dialog::SsidDialogMessage::SecuritySelected;
use crate::Message::InfoRow;
use iced::widget::button::Status::Hovered;
use std::sync::LazyLock;

const INFO_TEXT: &str = "To configure the Wi-Fi of the USB connected 'porky' device, complete the fields below and then press 'Send'";

static INPUT_ID: LazyLock<text_input::Id> = LazyLock::new(text_input::Id::unique);

#[derive(Debug, Clone)]
pub struct SsidDialog {
    ssid_spec: SsidSpec,
    error: String,
    pub show_modal: bool,
    show_spinner: bool,
    disable_widgets: bool,
    hardware_details: HardwareDetails,
}

#[derive(Clone, Debug)]
pub enum SsidDialogMessage {
    NameEntered(String),
    PasswordEntered(String),
    SecuritySelected(SSIDSecurity),
    ModalKeyEvent(Event),
    SendButtonPressed(String, String, SSIDSecurity),
    HideSsidDialog,
    ShowSsidDialog(HardwareDetails, Option<SsidSpec>),
    ConnectionError(String),
}

impl Default for SsidDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum SSIDSecurity {
    OPEN,
    WPA,
    WPA2,
    WPA3,
}

impl Display for SSIDSecurity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SSIDSecurity::OPEN => "open",
            SSIDSecurity::WPA => "wpa",
            SSIDSecurity::WPA2 => "wpa2",
            SSIDSecurity::WPA3 => "wpa3",
        })
    }
}

#[allow(unused_variables)]
fn send_ssid(serial_number: String, ssid_spec: SsidSpec) -> Task<Message> {
    #[cfg(feature = "usb-raw")]
    return Task::perform(
        usb_raw::send_ssid_spec(serial_number, ssid_spec),
        |res| match res {
            Ok(_) => InfoRow(ShowStatusMessage(Info("Wi-Fi Setup sent via USB".into()))),
            Err(e) => InfoRow(ShowStatusMessage(Error(
                "Error sending Wi-Fi Setup via USB".into(),
                e,
            ))),
        },
    );
    #[cfg(not(feature = "usb-raw"))]
    Task::none()
}

impl SsidSpec {
    /// Try and create a new [SsidSpec] using name, password and security fields, validating
    /// the combination. Return an `Ok` with the [SsisSpec] or an `Err` with an error string
    /// describing the cause of it being invalid.
    fn try_new(name: String, pass: String, security: SSIDSecurity) -> Result<SsidSpec, String> {
        if name.trim().is_empty() {
            return Err("Please Enter SSID name".into());
        }

        if name.trim().len() > SSID_NAME_LENGTH {
            return Err("SSID name is too long".into());
        }

        match security {
            SSIDSecurity::OPEN => {}
            SSIDSecurity::WPA | SSIDSecurity::WPA2 | SSIDSecurity::WPA3 => {
                if pass.trim().is_empty() {
                    return Err("Please Enter SSID password".into());
                }

                if pass.trim().len() > SSID_PASS_LENGTH {
                    return Err("SSID password is too long".into());
                }
            }
        }

        Ok(SsidSpec {
            ssid_name: name,
            ssid_pass: pass,
            ssid_security: security.to_string(),
        })
    }
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
                        self.error = error;
                        self.show_spinner = false;
                        self.disable_widgets = false;
                        Task::none()
                    }
                }
            }

            ShowSsidDialog(hardware_details, ssid_spec) => {
                self.hardware_details = hardware_details;
                self.ssid_spec = ssid_spec.unwrap_or(SsidSpec::default());
                self.show_modal = true;
                text_input::focus(INPUT_ID.clone())
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
        }
    }

    //noinspection RsLift
    pub fn view(&self) -> Element<'_, Message> {
        match self.disable_widgets {
            true => self.create_container(false),
            false => self.create_container(true),
        }
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

    fn send_row(&self) -> Row<'_, Message> {
        let mut cancel_button = Button::new(Text::new("Cancel")).style(|_, status| {
            if status == Hovered {
                MODAL_CANCEL_BUTTON_HOVER_STYLE
            } else {
                MODAL_CANCEL_BUTTON_STYLE
            }
        });

        let mut send_button = Button::new(Text::new("Send")).style(move |_theme, status| {
            if status == Hovered {
                MODAL_CONNECT_BUTTON_HOVER_STYLE
            } else {
                MODAL_CONNECT_BUTTON_STYLE
            }
        });

        let mut row = Row::new().align_y(iced::Alignment::Center);

        if self.show_spinner {
            row = row
                .push(cancel_button)
                .push(
                    Circular::new()
                        .easing(&EMPHASIZED_ACCELERATE)
                        .cycle_duration(Duration::from_secs_f32(2.0)),
                )
                .spacing(150);
        } else {
            cancel_button = cancel_button.on_press(Message::SsidDialog(HideSsidDialog));
            send_button = send_button.on_press(Message::SsidDialog(SendButtonPressed(
                self.ssid_spec.ssid_name.clone(),
                self.ssid_spec.ssid_pass.clone(),
                SSIDSecurity::OPEN, // TODO
            )));
            row = row.push(cancel_button).spacing(350);
        }

        row.push(send_button)
    }

    fn create_text_container(&self) -> Element<'_, Message> {
        container(Text::new(INFO_TEXT).style(move |_theme| INFO_TEXT_STYLE))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme| TEXT_BOX_CONTAINER_STYLE)
            .into()
    }

    //noinspection RsLiveness
    fn create_container(&self, input_enabled: bool) -> Element<'_, Message> {
        container(
            column![
                text("Configure Wi-Fi of USB connected 'porky' device").size(20),
                column![
                    self.create_text_container(),
                    text(self.error.clone()).style(move |_theme| { CONNECTION_ERROR_DISPLAY }),
                    text("SSID Name").size(12),
                    {
                        let mut name_input =
                            text_input("Enter SSID Name", &self.ssid_spec.ssid_name)
                                .padding(5)
                                .id(INPUT_ID.clone())
                                .on_submit(Message::SsidDialog(SendButtonPressed(
                                    self.ssid_spec.ssid_name.clone(),
                                    self.ssid_spec.ssid_pass.clone(),
                                    SSIDSecurity::OPEN, // TODO
                                )));
                        if input_enabled {
                            name_input = name_input
                                .on_input(|input| Message::SsidDialog(NameEntered(input)));
                        }
                        name_input
                    }
                ]
                .spacing(10),
                column![text("SSID Password").size(12), {
                    let mut password_input =
                        text_input("Enter SSID Password", &self.ssid_spec.ssid_pass)
                            .padding(5)
                            .on_submit(Message::SsidDialog(SendButtonPressed(
                                self.ssid_spec.ssid_name.clone(),
                                self.ssid_spec.ssid_pass.clone(),
                                SSIDSecurity::OPEN, // TODO
                            )));
                    if input_enabled {
                        password_input = password_input
                            .on_input(|input| Message::SsidDialog(PasswordEntered(input)));
                    }
                    password_input
                }]
                .spacing(5),
                self.send_row(),
            ]
            .spacing(10),
        )
        .style(move |_theme| MODAL_CONTAINER_STYLE)
        .width(520)
        .padding(15)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::ssid_dialog::SSIDSecurity::OPEN;
    #[test]
    fn test_show_ssid_dialog() {
        let mut ssid_dialog = SsidDialog::new();
        assert!(!ssid_dialog.show_modal);

        let _ = ssid_dialog.update(ShowSsidDialog(
            HardwareDetails::default(),
            Some(SsidSpec::default()),
        ));
        assert!(ssid_dialog.show_modal);
    }

    #[test]
    fn test_show_empty_ssid_dialog() {
        let mut ssid_dialog = SsidDialog::new();
        assert!(!ssid_dialog.show_modal);

        let _ = ssid_dialog.update(ShowSsidDialog(HardwareDetails::default(), None));
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
        let _ = ssid_dialog.update(SendButtonPressed("".to_string(), "".to_string(), OPEN));
        assert_eq!(ssid_dialog.error, "Please Enter SSID name");
    }

    #[test]
    fn test_send_button_pressed_invalid_name() {
        let mut ssid_dialog = SsidDialog::new();
        let invalid_name = "invalid_name".to_string();
        let _ = ssid_dialog.update(SendButtonPressed(invalid_name, "".to_string(), OPEN));
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
