use crate::connect_dialog_handler::{
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_STYLE, MODAL_CONTAINER_STYLE,
};
use crate::views::hardware_view::HardwareView;
use crate::Message;
use iced::keyboard::key;
use iced::widget::{button, column, container, text, Row};
use iced::Subscription;
use iced::{keyboard, window, Color, Element, Event, Task};

#[derive(Default)]
pub struct DisplayModal {
    pub show_modal: bool,
    title: String,
    body: String,
    is_warning: bool,
}

#[derive(Clone, Debug)]
pub enum ModalMessage {
    HideModal,
    UnsavedChangesExitModal,
    HardwareDetailsModal,
    VersionModal,
    ExitApp,
    EscKeyEvent(Event),
}

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            title: String::new(), // Title of the modal
            body: String::new(),  // Body of the modal
            show_modal: false,
            is_warning: false,
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
                self.title = "Unsaved Changes".to_string();
                self.body =
                    "You have unsaved changes. Do you want to exit without saving?".to_string();
                Task::none()
            }

            // Display hardware information
            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Connected Hardware".to_string();
                self.body = hardware_view.hw_description().to_string();
                Task::none()
            }

            // Display piggui information
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Piggui".to_string();
                self.body = crate::views::version::version().to_string();
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
        let mut button_row = Row::new();
        let mut text_style = Color::new(0.447, 0.624, 0.812, 1.0);

        if self.is_warning {
            text_style = Color::new(0.988, 0.686, 0.243, 1.0);
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
                    .color(Color::new(0.447, 0.624, 0.812, 1.0)),
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
