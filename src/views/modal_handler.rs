use crate::file_helper::pick_and_load;
use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
use crate::views::hardware_view::HardwareView;
use crate::views::version::REPOSITORY;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{button, column, container, text, Row, Text};
use iced::{keyboard, window, Color, Command, Element, Event, Length};
use iced_futures::core::Alignment;
use iced_futures::Subscription;

pub struct DisplayModal {
    pub show_modal: bool,
    is_warning: bool,
    modal_type: Option<ModalType>,
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
pub enum ModalMessage {
    HideModal,
    UnsavedChangesExitModal,
    UnsavedLoadConfigChangesModal,
    LoadFile,
    HardwareDetailsModal,
    VersionModal,
    ExitApp,
    EscKeyEvent(Event),
    OpenRepoLink,
}

pub(crate) const MODAL_CANCEL_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::from_rgba(0.8, 0.0, 0.0, 1.0), // Gnome like Red background color
    text_color: Color::WHITE,
    hovered_bg_color: Color::from_rgba(0.9, 0.2, 0.2, 1.0), // Slightly lighter red when hovered
    hovered_text_color: Color::WHITE,
    border_radius: 2.0,
};

pub(crate) const MODAL_CONNECT_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::from_rgba(0.0, 1.0, 1.0, 1.0), // Cyan background color
    text_color: Color::BLACK,
    hovered_bg_color: Color::from_rgba(0.0, 0.8, 0.8, 1.0), // Darker cyan color when hovered
    hovered_text_color: Color::WHITE,
    border_radius: 2.0,
};

pub(crate) const MODAL_CONTAINER_STYLE: ContainerStyle = ContainerStyle {
    border_color: Color::WHITE,
    background_color: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
    border_radius: 2.0,
    border_width: 2.0,
};

const HYPERLINK_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgba(0.0, 0.3, 0.8, 1.0),
    border_radius: 2.0,
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::from_rgba(0.0, 0.0, 0.6, 1.0),
};

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            is_warning: false,
            modal_type: None,
        }
    }

    pub fn update(
        &mut self,
        message: ModalMessage,
        hardware_view: &HardwareView,
    ) -> Command<Message> {
        match message {
            ModalMessage::HideModal => {
                self.show_modal = false;
                Command::none()
            }

            // Display warning for unsaved changes
            ModalMessage::UnsavedChangesExitModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.modal_type = Some(ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes. Do you want to exit without saving?"
                        .to_string(),
                    load_config: false,
                });
                Command::none()
            }

            // Display hardware information
            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.modal_type = Some(ModalType::Info {
                    title: "About Connected Hardware".to_string(),
                    body: hardware_view.hw_description().to_string(),
                    is_version: false,
                });
                Command::none()
            }

            ModalMessage::LoadFile => {
                self.show_modal = false;
                Command::batch(vec![pick_and_load()])
            }

            ModalMessage::UnsavedLoadConfigChangesModal => {
                self.show_modal = true;
                self.modal_type = Some(ModalType::Warning {
                    title: "Unsaved Changes".to_string(),
                    body: "You have unsaved changes, loading a new config will overwrite them"
                        .to_string(),
                    load_config: true,
                });
                Command::none()
            }

            // Display piggui information
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.modal_type = Some(ModalType::Info {
                    title: "About Piggui".to_string(),
                    body: crate::views::version::version().to_string(),
                    is_version: true,
                });
                Command::none()
            }

            ModalMessage::OpenRepoLink => {
                if let Err(e) = webbrowser::open(REPOSITORY) {
                    eprintln!("failed to open project repository: {}", e);
                }
                Command::none()
            }

            // Exits the Application
            ModalMessage::ExitApp => window::close(window::Id::MAIN),

            // When Pressed `Esc` focuses on previous widget and hide modal
            ModalMessage::EscKeyEvent(Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            })) => {
                self.show_modal = false;
                Command::none()
            }
            _ => Command::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.modal_type {
            Some(ModalType::Warning {
                title,
                body,
                load_config,
            }) => {
                let mut button_row = Row::new();
                let text_style = TextStyle {
                    text_color: Color::new(0.988, 0.686, 0.243, 1.0),
                };

                if *load_config {
                    button_row = button_row
                        .push(
                            button("Continue and load a new config")
                                .on_press(Message::ModalHandle(ModalMessage::LoadFile))
                                .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                        )
                        .push(
                            button("Return to app")
                                .on_press(Message::ModalHandle(ModalMessage::HideModal))
                                .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                        )
                        .spacing(120);
                } else {
                    button_row = button_row
                        .push(
                            button("Exit without saving")
                                .on_press(Message::ModalHandle(ModalMessage::ExitApp))
                                .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                        )
                        .push(
                            button("Return to app")
                                .on_press(Message::ModalHandle(ModalMessage::HideModal))
                                .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                        )
                        .spacing(220);
                }

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(text_style.get_text_color()),
                        column![text(body.clone()),].spacing(10),
                        column![button_row].spacing(5),
                    ]
                    .spacing(10)]
                    .spacing(20),
                )
                .style(MODAL_CONTAINER_STYLE.get_container_style())
                .width(520)
                .padding(15)
                .into()
            }
            Some(ModalType::Info {
                title,
                body,
                is_version,
            }) => {
                let text_style = TextStyle {
                    text_color: Color::new(0.447, 0.624, 0.812, 1.0),
                };
                let mut hyperlink_row = Row::new().width(Length::Fill);
                let mut button_row = Row::new();
                if *is_version {
                    hyperlink_row = hyperlink_row.push(Text::new("Full source available at: "));
                    hyperlink_row = hyperlink_row
                        .push(
                            button(Text::new("github"))
                                .on_press(Message::ModalHandle(ModalMessage::OpenRepoLink))
                                .style(HYPERLINK_BUTTON_STYLE.get_button_style()),
                        )
                        .align_items(Alignment::Center);
                    button_row = button_row.push(hyperlink_row);
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::ModalHandle(ModalMessage::HideModal))
                            .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                    );
                } else {
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::ModalHandle(ModalMessage::HideModal))
                            .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                    );
                }

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(text_style.get_text_color()),
                        column![text(body.clone()),].spacing(10),
                        column![button_row].spacing(5),
                    ]
                    .spacing(10)]
                    .spacing(20),
                )
                .style(MODAL_CONTAINER_STYLE.get_container_style())
                .width(520)
                .padding(15)
                .into()
            }
            None => container(column![]).into(), // Render empty container
        }
    }

    pub fn subscription(&self) -> Subscription<ModalMessage> {
        iced::event::listen().map(ModalMessage::EscKeyEvent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::hardware_view::HardwareView;

    #[test]
    fn test_hide_modal() {
        let mut display_modal = DisplayModal::new();
        display_modal.show_modal = true;
        display_modal.modal_type = Some(ModalType::Info {
            title: "Test".to_string(),
            body: "Test body".to_string(),
            is_version: false,
        });

        let _ = display_modal.update(ModalMessage::HideModal, &HardwareView::new());
        assert!(!display_modal.show_modal);
    }

    #[test]
    fn test_unsaved_changes_exit_modal() {
        let mut display_modal = DisplayModal::new();

        let _ = display_modal.update(ModalMessage::UnsavedChangesExitModal, &HardwareView::new());
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
        let mut display_modal = DisplayModal::new();

        let _ = display_modal.update(ModalMessage::VersionModal, &HardwareView::new());
        assert!(display_modal.show_modal);

        if let Some(ModalType::Info {
            title,
            body,
            is_version,
        }) = &display_modal.modal_type
        {
            assert_eq!(title, "About Piggui");
            assert_eq!(body, &crate::views::version::version().to_string());
        } else {
            panic!("ModalType should be Info");
        }
    }
}
