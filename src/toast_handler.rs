use crate::views::hardware_button::hw_description;
use crate::views::hardware_view::HardwareView;
use crate::widgets::toast::{Status, Toast};
use crate::Message;
use iced::Command;

#[derive(Debug, Clone)]
pub enum ToastMessage {
    VersionToast,
    HardwareDetailsToast,
    UnsavedChangesToast,
    UnsavedChangesExitToast,
    Close(usize),
    Timeout(f64),
}

pub struct ToastHandler {
    pub toasts: Vec<Toast>,
    pub showing_toast: bool,
    pub timeout_secs: u64,
    pending_load: bool,
}

impl ToastHandler {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            showing_toast: false,
            timeout_secs: crate::widgets::toast::DEFAULT_TIMEOUT,
            pending_load: false,
        }
    }

    pub fn update(
        &mut self,
        message: ToastMessage,
        hardware_view: Option<&HardwareView>,
    ) -> Command<Message> {
        match message {

            ToastMessage::UnsavedChangesExitToast => {
                self.toasts.clear();
                self.toasts.push(Toast {
                    title: "Unsaved Changes".into(),
                    body: "You have unsaved changes. Do you want to exit without saving?".into(),
                    status: Status::Danger,
                });
                self.showing_toast = true;
            }

            ToastMessage::UnsavedChangesToast => {
                self.toasts.clear();
                self.toasts.push(Toast {
                    title: "Unsaved Changes".into(),
                    body: "You have unsaved changes. Do you want to continue without saving?".into(),
                    status: Status::Danger,
                });
                self.showing_toast = true;
            }

            ToastMessage::VersionToast => {
                self.toasts.clear();
                self.toasts.push(Toast {
                    title: "About Piggui".into(),
                    body: crate::views::version::version(),
                    status: Status::Primary,
                });
                self.showing_toast = true;
            }

            ToastMessage::HardwareDetailsToast => {
                if self.showing_toast {
                    // Close the existing toast if `showing_toast` is true
                    let index = self.toasts.len() - 1;
                    return Command::perform(crate::empty(), move |_| {
                        Message::Toast(ToastMessage::Close(index))
                    });
                } else {
                    if let Some(hw_view) = hardware_view {
                        self.toasts.clear();
                        self.toasts.push(Toast {
                            title: "About Connected Hardware".into(),
                            body: hw_description(hw_view),
                            status: Status::Primary,
                        });
                        self.showing_toast = true;
                    }
                }
            }

            ToastMessage::Close(index) => {
                self.showing_toast = false;
                self.toasts.remove(index);
                if self.pending_load {
                    self.pending_load = false;
                    return crate::file_helper::pick_and_load();
                }
            }
            ToastMessage::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
            }
        }

        Command::none()
    }

    pub fn get_toasts(&self) -> &Vec<Toast> {
        &self.toasts
    }

    pub fn set_pending_load(&mut self, pending: bool) {
        self.pending_load = pending;
    }
}