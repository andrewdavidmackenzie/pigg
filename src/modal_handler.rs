use crate::views::hardware_view::HardwareView;
use crate::Message;
use iced::widget::{button, column, container, Text, text};
use iced::{Application, Color, Command, Element};
use crate::connect_dialog_handler::{MODAL_CANCEL_BUTTON_STYLE, MODAL_CONTAINER_STYLE};
use crate::styles::text_style::TextStyle;

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
    UnsavedChangesModal,
    UnsavedChangesExitModal,
    HardwareDetailsModal,
    VersionModal,
}

impl DisplayModal {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            show_modal: false,
            is_warning: false,
        }
    }
    pub fn update(
        &mut self,
        message: ModalMessage,
        hardware_view: &HardwareView,
    ) -> Command<Message> {
        match message {
            ModalMessage::ShowModal => {
                self.show_modal = true;
                return Command::none();
            }
            ModalMessage::HideModal => {
                self.show_modal = false;
                return Command::none();
            }
            ModalMessage::UnsavedChangesModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.title = "Unsaved Changes".to_string();
                self.body = "You have unsaved changes. Do you want to exit without saving?".to_string();
                return Command::none();
            }

            ModalMessage::UnsavedChangesExitModal => {
                self.show_modal = true;
                self.is_warning = true;
                self.title = "Unsaved Changes".to_string();
                self.body = "You have unsaved changes. Do you want to exit without saving?".to_string();
                return Command::none();
            }

            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Connected Hardware".to_string();
                self.body = hardware_view.hw_description().to_string();
                return Command::none();
            }
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.is_warning = false;
                self.title = "About Piggui".to_string();
                self.body = crate::views::version::version().to_string();
                return Command::none();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut text_style = TextStyle {
            text_color: Color::new(0.447, 0.624, 0.812, 1.0),
        };
        if self.is_warning {
            text_style = TextStyle {
                text_color: Color::new(0.988, 0.686, 0.243, 1.0),
            };
        }
        container(
            column![column![
                    text(self.title.clone()).size(20).style(text_style.get_text_color()),
                    column! [
                    text(self.body.clone()).size(14),
                ].spacing(10),
                    column![
                        button("Cancel").on_press(Message::ModalHandle(ModalMessage::HideModal)).style(MODAL_CANCEL_BUTTON_STYLE.get_button_style()),
                    ]
                    .spacing(5),
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
