use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::styles::text_style::TextStyle;
use crate::views::hardware_view::HardwareView;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{button, column, container, text, Row};
use iced::{keyboard, window, Color, Command, Element, Event};
use iced_futures::Subscription;
use crate::file_helper::pick_and_load;

pub struct DisplayModal {
    pub show_modal: bool,
    title: String,
    body: String,
    is_warning: bool,
    is_load: bool,
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

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            title: String::new(), // Title of the modal
            body: String::new(),  // Body of the modal
            show_modal: false,
            is_warning: false,
            is_load: false,
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
                self.is_load = false;
                self.title = "Unsaved Changes".to_string();
                self.body =
                    "You have unsaved changes. Do you want to exit without saving?".to_string();
                Command::none()
            }

            // Display hardware information
            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.is_load = false;
                self.title = "About Connected Hardware".to_string();
                self.body = hardware_view.hw_description().to_string();
                Command::none()
            }

            ModalMessage::LoadFile => {
                self.show_modal = false;
                return Command::batch(vec![pick_and_load()])
            }

            ModalMessage::UnsavedLoadConfigChangesModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.is_load = true;
                self.title = "Unsaved Changes".to_string();
                self.body = "You have unsaved changes, loading a new config will overwrite them".to_string();
                Command::none()

            }

            // Display piggui information
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.is_load = false;
                self.title = "About Piggui".to_string();
                self.body = crate::views::version::version().to_string();
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
        let mut button_row = Row::new();
        let mut text_style = TextStyle {
            text_color: Color::new(0.447, 0.624, 0.812, 1.0),
        };

        if self.is_warning && !self.is_load {
            text_style = TextStyle {
                text_color: Color::new(0.988, 0.686, 0.243, 1.0),
            };
            button_row = button_row.push(
                button("Exit without saving")
                    .on_press(Message::ModalHandle(ModalMessage::ExitApp))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            ); // Exits the application
            button_row = button_row
                .push(
                    button("Return to app")
                        .on_press(Message::ModalHandle(ModalMessage::HideModal))
                        .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                )
                .spacing(220);
        } else if self.is_warning && self.is_load {
            text_style = TextStyle {
                text_color: Color::new(0.988, 0.686, 0.243, 1.0),
            };
            button_row = button_row.push(
                button("Continue and load a new config")
                    .on_press(Message::ModalHandle(ModalMessage::LoadFile))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            );
            button_row = button_row
                .push(
                    button("Return to app")
                        .on_press(Message::ModalHandle(ModalMessage::HideModal))
                        .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                )
                .spacing(120);
        } else {
            button_row = button_row.push(
                button("Close")
                    .on_press(Message::ModalHandle(ModalMessage::HideModal))
                    .style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
            );
        }

        container(
            column![column![
                text(self.title.clone())
                    .size(20)
                    .style(text_style.get_text_color()),
                column![text(self.body.clone()),].spacing(10),
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

        let _ = display_modal.update(ModalMessage::HideModal, &HardwareView::new());
        assert!(!display_modal.show_modal);
        assert!(display_modal.title.is_empty());
        assert!(display_modal.body.is_empty());
        assert!(!display_modal.is_warning);
    }

    #[test]
    fn test_unsaved_changes_exit_modal() {
        let mut display_modal = DisplayModal::new();

        let _ = display_modal.update(ModalMessage::UnsavedChangesExitModal, &HardwareView::new());
        assert!(display_modal.show_modal);
        assert!(display_modal.is_warning);
        assert_eq!(display_modal.title, "Unsaved Changes");
        assert_eq!(
            display_modal.body,
            "You have unsaved changes. Do you want to exit without saving?"
        );
    }

    #[test]
    fn test_version_modal() {
        let mut display_modal = DisplayModal::new();

        let _ = display_modal.update(ModalMessage::VersionModal, &HardwareView::new());
        assert!(display_modal.show_modal);
        assert!(!display_modal.is_warning);
        assert_eq!(display_modal.title, "About Piggui");
        assert_eq!(
            display_modal.body,
            crate::views::version::version().to_string()
        );
    }
}
