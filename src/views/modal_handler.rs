use crate::file_helper::pick_and_load;
use crate::views::hardware_view::HardwareView;
use crate::views::version::REPOSITORY;
use crate::Message;
use iced::border::Radius;
use iced::keyboard::key;
use iced::widget::{button, column, container, text, Row, Space, Text};
use iced::{keyboard, window, Background, Border, Color, Element, Event, Length, Shadow, Task};
use iced_aw::style::Status;
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

pub(crate) const MODAL_CANCEL_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.8, 0.0, 0.0, 1.0))),
    // bg_color: Color::from_rgba(0.8, 0.0, 0.0, 1.0), // Gnome like Red background color
    text_color: Color::WHITE,
    border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: Radius {
            top_left: 2.0,
            top_right: 2.0,
            bottom_right: 2.0,
            bottom_left: 2.0,
        },
    },
    // hovered_bg_color: Color::from_rgba(0.9, 0.2, 0.2, 1.0), // Slightly lighter red when hovered
    // hovered_text_color: Color::WHITE,
    // border_radius: 2.0,
    shadow: Shadow {
        color: Color::TRANSPARENT,
        offset: iced::Vector { x: 0.0, y: 0.0 },
        blur_radius: 0.0,
    },
};

pub(crate) const MODAL_CONNECT_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::from_rgba(0.0, 1.0, 1.0, 1.0))),
    // bg_color: Color::from_rgba(0.0, 1.0, 1.0, 1.0), // Cyan background color
    text_color: Color::BLACK,
    border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: Radius {
            top_left: 2.0,
            top_right: 2.0,
            bottom_right: 2.0,
            bottom_left: 2.0,
        },
    },
    // hovered_bg_color: Color::from_rgba(0.0, 0.8, 0.8, 1.0), // Darker cyan color when hovered
    // hovered_text_color: Color::WHITE,
    // border_radius: 2.0,
    shadow: Shadow {
        color: Color::TRANSPARENT,
        offset: iced::Vector { x: 0.0, y: 0.0 },
        blur_radius: 0.0,
    },
};

pub(crate) const MODAL_CONTAINER_STYLE: container::Style = container::Style {
    text_color: Some(Color::BLACK),
    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 1.0))),
    border: Border {
        color: Color::WHITE,
        width: 2.0,
        radius: Radius {
            top_left: 2.0,
            top_right: 2.0,
            bottom_right: 2.0,
            bottom_left: 2.0,
        },
    },
    // border_color: Color::WHITE,
    // // background_color: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
    // border_radius: 2.0,
    // border_width: 2.0,
    shadow: Shadow {
        color: Color::TRANSPARENT,
        offset: iced::Vector { x: 0.0, y: 0.0 },
        blur_radius: 0.0,
    },
};

const HYPERLINK_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    // bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgba(0.0, 0.3, 0.8, 1.0),
    border: Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: Radius {
            top_left: 2.0,
            top_right: 2.0,
            bottom_right: 2.0,
            bottom_left: 2.0,
        },
    },
    // border_radius: 2.0,
    // hovered_bg_color: Color::TRANSPARENT,
    // hovered_text_color: Color::from_rgba(0.0, 0.0, 0.6, 1.0),
    shadow: Shadow {
        color: Color::TRANSPARENT,
        offset: iced::Vector { x: 0.0, y: 0.0 },
        blur_radius: 0.0,
    },
};

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            is_warning: false,
            modal_type: None,
        }
    }

    pub fn update(&mut self, message: ModalMessage, hardware_view: &HardwareView) -> Task<Message> {
        match message {
            ModalMessage::HideModal => {
                self.show_modal = false;
                Task::none()
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
                Task::none()
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
                Task::none()
            }

            ModalMessage::LoadFile => {
                self.show_modal = false;
                Task::batch(vec![pick_and_load()])
            }

            ModalMessage::UnsavedLoadConfigChangesModal => {
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
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.modal_type = Some(ModalType::Info {
                    title: "About Piggui".to_string(),
                    body: crate::views::version::version().to_string(),
                    is_version: true,
                });
                Task::none()
            }

            ModalMessage::OpenRepoLink => {
                if let Err(e) = webbrowser::open(REPOSITORY) {
                    eprintln!("failed to open project repository: {}", e);
                }
                Task::none()
            }

            // Exits the Application
            ModalMessage::ExitApp => window::close(window::Id::MAIN),

            // When Pressed `Esc` focuses on previous widget and hide modal
            ModalMessage::EscKeyEvent(Event::Keyboard(keyboard::Event::KeyPressed {
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
                let mut button_row = Row::new();
                let text_style = text::Style {
                    color: Some(Color::new(0.988, 0.686, 0.243, 1.0)),
                };

                if *load_config {
                    button_row = button_row
                        .push(
                            button("Continue and load a new config")
                                .on_press(Message::ModalHandle(ModalMessage::LoadFile))
                                .style(move |theme, status| MODAL_CANCEL_BUTTON_STYLE),
                        )
                        .push(
                            button("Return to app")
                                .on_press(Message::ModalHandle(ModalMessage::HideModal))
                                .style(move |theme, status| MODAL_CONNECT_BUTTON_STYLE),
                        )
                        .spacing(120);
                } else {
                    button_row = button_row
                        .push(
                            button("Exit without saving")
                                .on_press(Message::ModalHandle(ModalMessage::ExitApp))
                                .style(move |theme, status| MODAL_CANCEL_BUTTON_STYLE),
                        )
                        .push(Space::new(235, 10))
                        .push(
                            button("Return to app")
                                .on_press(Message::ModalHandle(ModalMessage::HideModal))
                                .style(move |theme, status| MODAL_CONNECT_BUTTON_STYLE),
                        )
                }

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(move |theme| { text_style }),
                        column![text(body.clone()),].spacing(10),
                        column![button_row].spacing(5),
                    ]
                    .spacing(10)]
                    .spacing(20),
                )
                .style(move |theme| MODAL_CONTAINER_STYLE)
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
                    color: Some(Color::new(0.447, 0.624, 0.812, 1.0)),
                };
                let mut hyperlink_row = Row::new().width(Length::Fill);
                let mut button_row = Row::new();
                if *is_version {
                    hyperlink_row = hyperlink_row.push(Text::new("Full source available at: "));
                    hyperlink_row = hyperlink_row
                        .push(
                            button(Text::new("github"))
                                .on_press(Message::ModalHandle(ModalMessage::OpenRepoLink))
                                .style(move |theme, status| HYPERLINK_BUTTON_STYLE),
                        )
                        .align_y(Alignment::Center);
                    button_row = button_row.push(hyperlink_row);
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::ModalHandle(ModalMessage::HideModal))
                            .style(move |theme, status| MODAL_CANCEL_BUTTON_STYLE),
                    );
                } else {
                    button_row = button_row.push(
                        button("Close")
                            .on_press(Message::ModalHandle(ModalMessage::HideModal))
                            .style(move |theme, status| MODAL_CANCEL_BUTTON_STYLE),
                    );
                }

                container(
                    column![column![
                        text(title.clone())
                            .size(20)
                            .style(move |theme, _status: Status| { text_style }),
                        column![text(body.clone()),].spacing(10),
                        column![button_row].spacing(5),
                    ]
                    .spacing(10)]
                    .spacing(20),
                )
                .style(move |theme, _status: Status| MODAL_CONTAINER_STYLE)
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
            assert!(is_version);
            assert_eq!(title, "About Piggui");
            assert_eq!(body, &crate::views::version::version().to_string());
        } else {
            panic!("ModalType should be Info");
        }
    }
}
