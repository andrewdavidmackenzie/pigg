use crate::connect_dialog_handler::{
    MODAL_CANCEL_BUTTON_STYLE, MODAL_CONNECT_BUTTON_STYLE, MODAL_CONTAINER_STYLE,
};
use crate::styles::text_style::TextStyle;
use crate::views::hardware_view::HardwareView;
use crate::Message;
use iced::widget::{button, column, container, text, Row};
use iced::{window, Color, Command, Element};

pub struct DisplayModal {
    pub show_modal: bool,
    title: String,
    body: String,
    is_warning: bool,
}

#[derive(Clone, Debug)]
pub enum ModalMessage {
    ShowModal,
    HideModal,
    UnsavedChangesExitModal,
    HardwareDetailsModal,
    VersionModal,
    ExitApp,
}

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            title: String::new(), // Title of the modal
            body: String::new(), // Body of the modal
            show_modal: false,
            is_warning: false,
        }
    }
    pub fn update(
        &mut self,
        message: ModalMessage,
        hardware_view: &HardwareView,
    ) -> Command<Message> {
        return match message {
            ModalMessage::ShowModal => {
                self.show_modal = true;
                Command::none()
            }
            ModalMessage::HideModal => {
                self.show_modal = false;
                Command::none()
            }

            // Display warning for unsaved changes
            ModalMessage::UnsavedChangesExitModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.title = "Unsaved Changes".to_string();
                self.body =
                    "You have unsaved changes. Do you want to exit without saving?".to_string();
                Command::none()
            }

            // Display hardware information
            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Connected Hardware".to_string();
                self.body = hardware_view.hw_description().to_string();
                Command::none()
            }

            // Display piggui information
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Piggui".to_string();
                self.body = crate::views::version::version().to_string();
                Command::none()
            }

            // Exits the Application
            ModalMessage::ExitApp => {
                window::close(window::Id::MAIN)
            }
        };
    }

    pub fn view(&self) -> Element<Message> {
        let mut button_row = Row::new();
        let mut text_style = TextStyle {
            text_color: Color::new(0.447, 0.624, 0.812, 1.0),
        };
        if self.is_warning {
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
                    button("Save")
                        .on_press(Message::ModalHandle(ModalMessage::HideModal))
                        .style(MODAL_CONNECT_BUTTON_STYLE.get_button_style()),
                )
                .spacing(290);
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
                column![text(self.body.clone()).size(14),].spacing(10),
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
}
