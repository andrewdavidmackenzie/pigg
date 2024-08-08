use crate::views::hardware_view::HardwareView;
use crate::Message;
use iced::widget::{button, column, container, Text, text};
use iced::{Command, Element};

pub struct DisplayModal {
    pub show_modal: bool,
    title: String,
    body: String,
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
                self.title = "Unsaved Changes".to_string();
                self.body = "You have unsaved changes. Do you want to exit without saving?".to_string();
                return Command::none();
            }

            ModalMessage::UnsavedChangesExitModal => {
                self.show_modal = true;
                self.title = "Unsaved Changes".to_string();
                self.body = "You have unsaved changes. Do you want to exit without saving?".to_string();
                return Command::none();
            }

            ModalMessage::HardwareDetailsModal => {
                self.show_modal = true;
                self.title = "About connected Hardware".to_string();
                self.body = hardware_view.hw_description().to_string();
                return Command::none();
            }
            ModalMessage::VersionModal => {
                self.show_modal = true;
                self.title = "Version is".to_string();
                self.body = crate::views::version::version().to_string();
                return Command::none();
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        container(
            column![column![
                    text("Modal").size(20),
                    column![
                        text(self.title.clone()).size(12),
                    ]
                    .spacing(10),
                    column! [
                    text(self.body.clone()).size(12),
                ].spacing(10),
                    column![
                        button("Cancel").on_press(Message::ModalHandle(ModalMessage::HideModal)),
                    ]
                    .spacing(5),
                ]
                .spacing(10)]
                .spacing(20),
        )
            .width(520)
            .padding(15)
            .into()
    }
}
