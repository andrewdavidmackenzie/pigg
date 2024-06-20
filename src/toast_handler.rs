use crate::views::hardware_button::hw_description;
use crate::views::hardware_view::HardwareView;
use crate::widgets::toast::{Manager, Status, Toast};
use crate::Message;
use iced::{Command, Element};

#[derive(Debug, Clone)]
pub enum ToastMessage {
    VersionToast,
    HardwareDetailsToast,
    UnsavedChangesToast,
    UnsavedChangesExitToast,
    Close(usize),
    CloseLastToast,
    Timeout(f64),
}

pub struct ToastHandler {
    toasts: Vec<Toast>,
    showing_toast: bool,
    timeout_secs: u64,
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
                self.clear_toasts();
                self.push_toast(Toast {
                    title: "Unsaved Changes".into(),
                    body: "You have unsaved changes. Do you want to exit without saving?".into(),
                    status: Status::Danger,
                });
                self.showing_toast = true;
            }

            ToastMessage::UnsavedChangesToast => {
                self.clear_toasts();
                self.push_toast(Toast {
                    title: "Unsaved Changes".into(),
                    body: "You have unsaved changes. Do you want to continue without saving?"
                        .into(),
                    status: Status::Danger,
                });
                self.showing_toast = true;
                self.set_pending_load(true);
            }

            ToastMessage::VersionToast => {
                self.clear_toasts();
                self.push_toast(Toast {
                    title: "About Piggui".into(),
                    body: crate::views::version::version(),
                    status: Status::Primary,
                });
                self.showing_toast = true;
            }

            ToastMessage::HardwareDetailsToast => {
                if self.showing_toast {
                    // Close the existing toast if `showing_toast` is true
                    if let Some(index) = self.get_latest_toast_index() {
                        return Command::perform(crate::empty(), move |_| {
                            Message::Toast(ToastMessage::Close(index))
                        });
                    }
                } else if let Some(hw_view) = hardware_view {
                    self.clear_toasts();
                    self.push_toast(Toast {
                        title: "About Connected Hardware".into(),
                        body: hw_description(hw_view),
                        status: Status::Primary,
                    });
                    self.showing_toast = true;
                }
            }

            ToastMessage::Close(index) => {
                self.showing_toast = false;
                self.remove_toast(index);
                if self.pending_load {
                    self.pending_load = false;
                    return crate::file_helper::pick_and_load();
                }
            }

            ToastMessage::CloseLastToast => {
                if let Some(index) = self.get_latest_toast_index() {
                    self.showing_toast = false;
                    self.remove_toast(index);
                    if self.pending_load {
                        self.pending_load = false;
                        return crate::file_helper::pick_and_load();
                    }
                }
            }

            ToastMessage::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
            }
        }

        Command::none()
    }

    /// Take an element and wrap it in a Manager for toasts
    pub fn view<'a>(&'a self, content: Element<'a, Message>) -> Element<'a, Message> {
        Manager::new(content, &self.toasts, |index| {
            Message::Toast(ToastMessage::Close(index))
        })
        .timeout(self.timeout_secs)
        .into()
    }

    fn clear_toasts(&mut self) {
        self.toasts.clear();
    }

    fn push_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    fn remove_toast(&mut self, index: usize) {
        if index < self.toasts.len() {
            self.toasts.remove(index);
        }
    }

    fn get_latest_toast_index(&self) -> Option<usize> {
        if !self.toasts.is_empty() {
            Some(self.toasts.len() - 1)
        } else {
            None
        }
    }

    fn set_pending_load(&mut self, pending: bool) {
        self.pending_load = pending;
    }

    pub fn is_showing_toast(&self) -> bool {
        self.showing_toast
    }

    #[cfg(test)]
    pub fn get_toasts(&self) -> &Vec<Toast> {
        &self.toasts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_toast_message() {
        let mut toast_handler = ToastHandler::new();

        // No toasts should be present
        assert!(toast_handler.get_toasts().is_empty());

        // Add a toast
        let _ = toast_handler.update(ToastMessage::VersionToast, None);

        // Check if a toast was added
        assert_eq!(toast_handler.get_toasts().len(), 1);
        let toast = &toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "About Piggui");
    }

    #[test]
    fn test_close_toast_message() {
        let mut toast_handler = ToastHandler::new();

        // Add a toast
        let _ = toast_handler.update(ToastMessage::VersionToast, None);

        // Ensure the toast was added
        assert_eq!(toast_handler.get_toasts().len(), 1);

        // Close the toast
        let _ = toast_handler.update(ToastMessage::Close(0), None);

        // Check if the toast was removed
        assert!(toast_handler.get_toasts().is_empty());
    }

    #[test]
    fn test_toast_timeout() {
        let mut toast_handler = ToastHandler::new();

        // Send a timeout message
        let _ = toast_handler.update(ToastMessage::Timeout(5.0), None);

        // Check the timeout
        assert_eq!(toast_handler.get_timeout(), 5);
    }

    #[test]
    fn test_pending_load_toast() {
        let mut toast_handler = ToastHandler::new();

        // Add a toast
        let _ = toast_handler.update(ToastMessage::VersionToast, None);
        assert!(toast_handler.is_showing_toast());

        // Set pending load
        toast_handler.set_pending_load(true);

        // Close the toast
        let _ = toast_handler.update(ToastMessage::Close(0), None);

        // Pending load should be false after closing the toast
        assert!(!toast_handler.pending_load);
    }
}
