use crate::file_helper::pick_and_load;
use crate::hw_definition::description::HardwareDetails;
use crate::views::dialog_styles::{
    HYPERLINK_BUTTON_HOVER_STYLE, HYPERLINK_BUTTON_STYLE, MODAL_CANCEL_BUTTON_HOVER_STYLE,
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_HOVER_STYLE, MODAL_CONNECT_BUTTON_STYLE,
    MODAL_CONTAINER_STYLE,
};
use crate::views::hardware_view::HardwareConnection;
#[cfg(feature = "tcp")]
use crate::views::hardware_view::HardwareConnection::Tcp;
use crate::views::version::REPOSITORY;
use crate::Message;
use iced::keyboard::key;
use iced::widget::button::Status::Hovered;
use iced::widget::{button, column, container, horizontal_space, text, Row, Space, Text};
use iced::{keyboard, window, Color, Element, Event, Length, Task};
use iced_futures::core::Alignment;
use iced_futures::Subscription;
#[cfg(feature = "tcp")]
use std::net::IpAddr;

pub struct InfoDialog {
    pub show_modal: bool,
    is_warning: bool,
    modal_type: Option<ModalType>,
    target: Option<HardwareConnection>,
}
pub enum ModalType {
    Warning {
        title: String,
        body: String,
        load_config: bool,
    },
    Info {
        title: String,
        body: String,
        is_version: bool,
    },
}

#[derive(Clone, Debug)]
pub enum InfoDialogMessage {
    HideModal,
    UnsavedChangesExitModal,
    UnsavedLoadConfigChangesModal,
    LoadFile,
    HardwareDetailsModal(HardwareDetails, Option<(IpAddr, u16)>),
    AboutDialog,
    ExitApp,
    EscKeyEvent(Event),
    OpenRepoLink,
}

impl InfoDialog {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            is_warning: false,
            modal_type: None,
            target: None,
        }
    }

    pub fn update(&mut self, message: InfoDialogMessage) -> Task<Message> {
        match message {
            InfoDialogMessage::HideModal => {
                self.show_modal = false;
                Task::none()
            }

            // Display warning for unsaved changes
            InfoDialogMessage::UnsavedChangesExitModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.modal_type = Some(ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes. Do you want to exit without saving?"
                        .to_string(),
                    load_config: false,
                });
                Task::none()
            }

            // Display hardware information
            #[allow(unused_variables)]
            InfoDialogMessage::HardwareDetailsModal(hardware_details, tcp) => {
                self.show_modal = true;
                self.is_warning = false;
                #[allow(unused_mut)]
                let mut body = format!("{hardware_details}");

                #[cfg(feature = "tcp")]
                if let Some(t) = tcp {
                    self.target = Some(Tcp(t.0, t.1));
                    body.push_str(&format!("\nIP Address/Port: {}:{}", t.0, t.1));
                }

                self.modal_type = Some(ModalType::Info {
                    title: "Device Details".to_string(),
                    body,
                    is_version: false,
                });
                Task::none()
            }

            InfoDialogMessage::LoadFile => {
                self.show_modal = false;
                Task::batch(vec![pick_and_load()])
            }

            InfoDialogMessage::UnsavedLoadConfigChangesModal => {
                self.show_modal = true;
                self.modal_type = Some(ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes, loading a new config will overwrite them"
                        .to_string(),
                    load_config: true,
                });
                Task::none()
            }

            // Display piggui information
            InfoDialogMessage::AboutDialog => {
                self.show_modal = true;
                self.is_warning = false;
                self.modal_type = Some(ModalType::Info {
                    title: "About Piggui".to_string(),
                    body: crate::views::version::version().to_string(),
                    is_version: true,
                });
                Task::none()
            }

            InfoDialogMessage::OpenRepoLink => {
                if let Err(e) = webbrowser::open(REPOSITORY) {
                    eprintln!("failed to open project repository: {}", e);
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
                self.show_modal = false;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.modal_type {
            Some(ModalType::Warning {
                title,
                body,
                load_config,
            }) => {
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

                action_button = action_button.style(move |_theme, status| {
                    if status == Hovered {
                        MODAL_CANCEL_BUTTON_HOVER_STYLE
                    } else {
                        MODAL_CANCEL_BUTTON_STYLE
                    }
                });
                let mut button_row = Row::new().push(action_button);
                button_row = button_row.push(Space::new(Length::Fill, 10));
                button_row = button_row.push(
                    button("Return to app")
                        .on_press(Message::Modal(InfoDialogMessage::HideModal))
                        .style(move |_theme, status| {
                            if status == Hovered {
                                MODAL_CONNECT_BUTTON_HOVER_STYLE
                            } else {
                                MODAL_CONNECT_BUTTON_STYLE
                            }
                        }),
                );

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(move |_theme| { text_style }),
                        column![text(body.clone()),].spacing(10),
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
            Some(ModalType::Info {
                title,
                body,
                is_version,
            }) => {
                let text_style = text::Style {
                    color: Some(Color::WHITE),
                };
                let mut hyperlink_row = Row::new().width(Length::Fill);
                let mut button_row = Row::new();
                if *is_version {
                    hyperlink_row = hyperlink_row.push(Text::new("Full source available at: "));
                    hyperlink_row = hyperlink_row
                        .push(
                            button(Text::new("github"))
                                .on_press(Message::Modal(InfoDialogMessage::OpenRepoLink))
                                .style(move |_theme, status| {
                                    if status == Hovered {
                                        HYPERLINK_BUTTON_HOVER_STYLE
                                    } else {
                                        HYPERLINK_BUTTON_STYLE
                                    }
                                }),
                        )
                        .align_y(Alignment::Center);
                    button_row = button_row.push(hyperlink_row);
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::Modal(InfoDialogMessage::HideModal))
                            .style(move |_theme, status| {
                                if status == Hovered {
                                    MODAL_CANCEL_BUTTON_HOVER_STYLE
                                } else {
                                    MODAL_CANCEL_BUTTON_STYLE
                                }
                            }),
                    );
                } else {
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::Modal(InfoDialogMessage::HideModal))
                            .style(move |_theme, status| {
                                if status == Hovered {
                                    MODAL_CANCEL_BUTTON_HOVER_STYLE
                                } else {
                                    MODAL_CANCEL_BUTTON_STYLE
                                }
                            }),
                    );
                    if let Some(target) = &self.target {
                        button_row = button_row
                            .push(horizontal_space())
                            .push(
                                button("Connect")
                                    .on_press(Message::ConnectRequest(target.clone()))
                                    .style(move |_theme, status| {
                                        if status == Hovered {
                                            MODAL_CONNECT_BUTTON_HOVER_STYLE
                                        } else {
                                            MODAL_CONNECT_BUTTON_STYLE
                                        }
                                    }),
                            )
                            .align_y(Alignment::Center);
                    }
                }

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(move |_theme| { text_style }),
                        column![text(body.clone()),].spacing(10),
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
            None => container(column![]).into(), // Render empty container
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
        display_modal.show_modal = true;
        display_modal.modal_type = Some(ModalType::Info {
            title: "Test".to_string(),
            body: "Test body".to_string(),
            is_version: false,
        });

        let _ = display_modal.update(InfoDialogMessage::HideModal);
        assert!(!display_modal.show_modal);
    }

    #[test]
    fn test_unsaved_changes_exit_modal() {
        let mut display_modal = InfoDialog::new();

        let _ = display_modal.update(InfoDialogMessage::UnsavedChangesExitModal);
        assert!(display_modal.show_modal);

        if let Some(ModalType::Warning {
            title,
            body,
            load_config,
        }) = &display_modal.modal_type
        {
            assert_eq!(title, "Unsaved Changes");
            assert_eq!(
                body,
                "You have unsaved changes. Do you want to exit without saving?"
            );
            assert!(!*load_config);
        } else {
            panic!("ModalType should be Warning");
        }
    }

    #[test]
    fn test_version_modal() {
        let mut display_modal = InfoDialog::new();

        let _ = display_modal.update(InfoDialogMessage::AboutDialog);
        assert!(display_modal.show_modal);

        if let Some(ModalType::Info {
            title,
            body,
            is_version,
        }) = &display_modal.modal_type
        {
            assert!(is_version);
            assert_eq!(title, "About Piggui");
            assert_eq!(body, &crate::views::version::version().to_string());
        } else {
            panic!("ModalType should be Info");
        }
    }
}
