use crate::Message;
use iced::Task;

/// Check a series of conditions at that could cause problems - producing warning messages for
/// any issues found
pub fn run_preflight_checks() -> Task<Message> {
    Task::perform(checks(), |msg| msg)
}

/// perform he preflight checks asynchronously
async fn checks() -> Message {
    #[cfg(target_os = "linux")]
    return check_usb_permissions();
    #[cfg(not(target_os = "linux"))]
    Message::PreflightChecksDone
}
