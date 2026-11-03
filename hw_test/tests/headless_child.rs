//! Run a piggui connection in a short-lived headless child process.
//!
//! `iced_test::Emulator` runs the real application code, but it cannot release
//! OS resources such as claimed USB interfaces until the process exits. Each
//! connection therefore gets its own process. The process prints a
//! `HEADLESS_STATUS:` line and exits, which guarantees teardown.

use piggui::headless::Status;
use std::process::{Command, Stdio};

/// Run the `piggui_headless` helper with the given connection arguments and
/// return the status it reported.
///
/// The helper has its own timeout and always exits, so this call cannot block
/// indefinitely.
pub fn run_headless(args: &[String]) -> Status {
    let executable = env!("CARGO_BIN_EXE_piggui_headless");

    let output = Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|error| panic!("Failed to run '{executable}': {error}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if let Some(status) = parse_status(&stdout) {
        return status;
    }

    panic!(
        "The headless piggui helper did not report a status.\n\
         Arguments: {args:?}\n\
         Exit status: {}\n\
         Stdout:\n{stdout}\n\
         Stderr:\n{stderr}",
        output.status
    );
}

fn parse_status(stdout: &str) -> Option<Status> {
    for line in stdout.lines().rev() {
        let Some(status) = line.strip_prefix("HEADLESS_STATUS: ") else {
            continue;
        };

        return Some(match status {
            "CONNECTED" => Status::Connected,
            "TIMEOUT" => Status::ConnectionError("headless helper timed out".to_string()),
            "DISCONNECTED" => Status::Disconnected,
            other => match other.strip_prefix("CONNECTION_ERROR ") {
                Some(details) => Status::ConnectionError(details.to_string()),
                None => Status::ConnectionError(format!("Unexpected helper status: {other}")),
            },
        });
    }

    None
}
