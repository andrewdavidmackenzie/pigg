use crate::file_helper::pick_and_load;
use crate::views::about::REPOSITORY;
use crate::views::dialog_styles::{
    cancel_button, connect_button, hyperlink_button, MODAL_CONTAINER_STYLE,
};
use crate::views::hardware_styles::TOOLTIP_STYLE;
use crate::Message;
use iced::keyboard::key;
use iced::widget::tooltip::Position;
use iced::widget::{button, column, container, horizontal_space, text, Row, Space, Text, Tooltip};
use iced::{keyboard, window, Color, Element, Event, Length, Task};
use iced_futures::core::Alignment;
use iced_futures::Subscription;
use pigdef::description::HardwareDetails;
use pignet::HardwareConnection;
use std::collections::HashMap;

pub struct InfoDialog {
    modal_type: ModalType,
}

#[derive(PartialEq)]
pub enum ModalType {
    Error {
        title: &'static str,
        body: &'static str,
        help_link: &'static str,
    },
    Warning {
        title: String,
        body: String,
        load_config: bool,
    },
    DeviceDetails {
        title: String,
        body: String,
        hardware_connections: HashMap<String, HardwareConnection>,
    },
    Version {
        title: String,
        body: String,
    },
    None,
}

const WHITE_TEXT: text::Style = text::Style {
    color: Some(Color::WHITE),
};

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum InfoDialogMessage {
    HideModal,
    UnsavedChangesExitModal,
    UnsavedLoadConfigChangesModal,
    LoadFile,
    HardwareDetailsModal(HardwareDetails, HashMap<String, HardwareConnection>),
    AboutDialog,
    ErrorWithHelp(&'static str, &'static str, &'static str),
    ExitApp,
    EscKeyEvent(Event),
    OpenLink(&'static str),
}

impl InfoDialog {
    pub fn new() -> Self {
        Self {
            modal_type: ModalType::None,
        }
    }

    pub fn showing_modal(&self) -> bool {
        self.modal_type != ModalType::None
    }

    pub fn update(&mut self, message: InfoDialogMessage) -> Task<Message> {
        match message {
            InfoDialogMessage::HideModal => {
                self.modal_type = ModalType::None;
                Task::none()
            }

            // Display warning for unsaved changes
            InfoDialogMessage::UnsavedChangesExitModal => {
                self.modal_type = ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes. Do you want to exit without saving?"
                        .to_string(),
                    load_config: false,
                };
                Task::none()
            }

            // Display hardware information
            #[allow(unused_variables)]
            InfoDialogMessage::HardwareDetailsModal(hardware_details, hardware_connections) => {
                self.modal_type = ModalType::DeviceDetails {
                    title: "Device Details".to_string(),
                    body: format!("{hardware_details}\n"),
                    hardware_connections: hardware_connections.clone(),
                };
                Task::none()
            }

            InfoDialogMessage::LoadFile => {
                self.modal_type = ModalType::None;
                Task::batch(vec![pick_and_load()])
            }

            InfoDialogMessage::UnsavedLoadConfigChangesModal => {
                self.modal_type = ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes, loading a new config will overwrite them"
                        .to_string(),
                    load_config: true,
                };
                Task::none()
            }

            // Display piggui information
            InfoDialogMessage::AboutDialog => {
                self.modal_type = ModalType::Version {
                    title: "About Piggui".to_string(),
                    body: crate::views::about::about().to_string(),
                };
                Task::none()
            }

            InfoDialogMessage::ErrorWithHelp(title, body, help_link) => {
                self.modal_type = ModalType::Error {
                    title,
                    body,
                    help_link,
                };
                Task::none()
            }

            InfoDialogMessage::OpenLink(link) => {
                if let Err(e) = webbrowser::open(link) {
                    eprintln!("failed to open project repository: {e}");
                }
                Task::none()
            }

            // Exits the Application
            InfoDialogMessage::ExitApp => window::get_latest().and_then(window::close),

            // When Pressed `Esc` focuses on previous widget and hide modal
            InfoDialogMessage::EscKeyEvent(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            })) => {
                self.modal_type = ModalType::None;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn info_container<'a>(
        title: &'a str,
        body: &'a str,
        button_row: Row<'a, Message>,
        text_style: text::Style,
    ) -> Element<'a, Message> {
        container(
            column![column![
                text(title).size(20).style(move |_theme| { text_style }),
                column![text(body),].spacing(10),
                column![button_row].spacing(5),
            ]
            .spacing(10)]
            .spacing(20),
        )
        .style(move |_theme| MODAL_CONTAINER_STYLE)
        .width(520)
        .padding(15)
        .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.modal_type {
            ModalType::Warning {
                title,
                body,
                load_config,
            } => {
                let text_style = text::Style {
                    color: Some(Color::new(0.988, 0.686, 0.243, 1.0)),
                };

                let mut action_button = if *load_config {
                    button("Continue and load a new config")
                        .on_press(Message::Modal(InfoDialogMessage::LoadFile))
                } else {
                    button("Exit without saving")
                        .on_press(Message::Modal(InfoDialogMessage::ExitApp))
                };

                action_button = action_button.style(cancel_button);
                let mut button_row = Row::new().push(action_button);
                button_row = button_row.push(Space::new(Length::Fill, 10));
                button_row = button_row.push(
                    button("Return to app")
                        .on_press(Message::Modal(InfoDialogMessage::HideModal))
                        .style(connect_button),
                );

                Self::info_container(title, body, button_row, text_style)
            }

            ModalType::DeviceDetails {
                title,
                body,
                hardware_connections,
            } => {
                let mut button_row = Row::new();

                button_row = button_row.push(
                    button("Close")
                        .on_press(Message::Modal(InfoDialogMessage::HideModal))
                        .style(cancel_button),
                );
                for (name, hardware_connection) in hardware_connections {
                    let button = button(text(format!("Connect via {name}")))
                        .on_press(Message::ConnectRequest(hardware_connection.clone()))
                        .style(connect_button);
                    button_row = button_row
                        .push(horizontal_space())
                        .push(
                            Tooltip::new(
                                button,
                                text(format!("{hardware_connection}")),
                                Position::Top,
                            )
                            .gap(4.0)
                            .style(|_| TOOLTIP_STYLE),
                        )
                        .align_y(Alignment::Center);
                }

                Self::info_container(title, body, button_row, WHITE_TEXT)
            }

            ModalType::Version { title, body } => {
                let mut hyperlink_row = Row::new().width(Length::Fill);
                let mut button_row = Row::new();
                hyperlink_row = hyperlink_row.push(Text::new("Full source available at: "));
                hyperlink_row = hyperlink_row
                    .push(
                        button(Text::new("github"))
                            .on_press(Message::Modal(InfoDialogMessage::OpenLink(REPOSITORY)))
                            .style(hyperlink_button),
                    )
                    .align_y(Alignment::Center);
                button_row = button_row.push(hyperlink_row);
                button_row = button_row.push(
                    button("Close")
                        .on_press(Message::Modal(InfoDialogMessage::HideModal))
                        .style(cancel_button),
                );

                Self::info_container(title, body, button_row, WHITE_TEXT)
            }

            ModalType::Error {
                title,
                body,
                help_link,
            } => {
                let mut button_row = Row::new();
                let help_button = button(Text::new("Help"))
                    .on_press(Message::Modal(InfoDialogMessage::OpenLink(help_link)))
                    .style(hyperlink_button);
                button_row = button_row.push(help_button);
                button_row = button_row.push(
                    button("Close")
                        .on_press(Message::Modal(InfoDialogMessage::HideModal))
                        .style(cancel_button),
                );

                Self::info_container(title, body, button_row, WHITE_TEXT)
            }

            ModalType::None => container(column![]).into(), // Render empty container
        }
    }

    pub fn subscription(&self) -> Subscription<InfoDialogMessage> {
        iced::event::listen().map(InfoDialogMessage::EscKeyEvent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hide_modal() {
        let mut display_modal = InfoDialog::new();
        display_modal.modal_type = ModalType::DeviceDetails {
            title: "Test".to_string(),
            body: "Test body".to_string(),
            hardware_connections: HashMap::new(),
        };

        let _ = display_modal.update(InfoDialogMessage::HideModal);
        assert!(!display_modal.showing_modal());
    }

    #[test]
    fn test_unsaved_changes_exit_modal() {
        let mut display_modal = InfoDialog::new();

        let _ = display_modal.update(InfoDialogMessage::UnsavedChangesExitModal);

        match display_modal.modal_type {
            ModalType::Warning {
                title,
                body,
                load_config: _,
            } => {
                assert_eq!(title, "Unsaved Changes");
                assert_eq!(
                    body,
                    "You have unsaved changes. Do you want to exit without saving?"
                );
            }
            _ => {
                panic!("ModalType should be Warning");
            }
        }
    }

    #[test]
    fn test_version_modal() {
        let mut display_modal = InfoDialog::new();

        let _ = display_modal.update(InfoDialogMessage::AboutDialog);

        match display_modal.modal_type {
            ModalType::Version { title, body } => {
                assert_eq!(title, "About Piggui");
                assert_eq!(body, crate::views::about::about().to_string());
            }
            _ => {
                panic!("ModalType should be Info");
            }
        }
    }
}
