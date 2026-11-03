#![cfg(not(target_arch = "wasm32"))]

//! Shared helper for running the real Piggui application headlessly with
//! `iced_test::Emulator`.
//!
//! This lives in its own module (rather than `support.rs`) because `support.rs`
//! is also included by the `pigglet` integration tests, which do not depend on
//! the `piggui` library.

use std::time::{Duration, Instant};

use iced_test::core::Size;
use iced_test::emulator::{self, Emulator};
use iced_test::futures::futures::channel::mpsc;
use piggui::headless::{HeadlessApp, Status};

/// Run a headless Piggui app until it connects or fails to connect.
///
/// This executes the real boot, update, view, task and subscription code with
/// `iced_test::Emulator`, so the hardware connection path is exercised without
/// opening a host UI window.
///
/// When the app reaches a terminal state, any hardware connection is explicitly
/// disconnected before returning. Unlike a spawned GUI process, an in-process
/// emulator is not torn down when the test returns, so leaving the connection
/// open would keep claiming USB interfaces and Iroh endpoints and make later
/// tests fail.
#[allow(dead_code)]
pub fn run_headless_until_terminal(app: HeadlessApp, timeout: Duration) -> Status {
    type TestEmulator = Emulator<HeadlessApp>;

    fn pump(
        emulator: &mut TestEmulator,
        app: &HeadlessApp,
        receiver: &mut mpsc::Receiver<emulator::Event<HeadlessApp>>,
    ) {
        while let Ok(event) = receiver.try_recv() {
            match event {
                emulator::Event::Action(action) => emulator.perform(app, action),
                emulator::Event::Ready | emulator::Event::Failed(_) => {}
            }
        }
    }

    let (sender, mut receiver) = mpsc::channel(256);
    let mut emulator = Emulator::new(sender, &app, emulator::Mode::Zen, Size::new(1024.0, 768.0));

    let start = Instant::now();

    let status = loop {
        pump(&mut emulator, &app, &mut receiver);

        if app.ever_connected() {
            break Status::Connected;
        }

        let status = app.status();
        if matches!(status, Status::ConnectionError(_)) || start.elapsed() > timeout {
            break status;
        }

        std::thread::sleep(Duration::from_millis(10));
    };

    // Disconnect explicitly so the hardware subscription drops the connection
    // (and releases the device) before the emulator is dropped.
    if matches!(status, Status::Connected) {
        emulator.update(&app, piggui::Message::Disconnect);

        let disconnect_start = Instant::now();
        while disconnect_start.elapsed() < Duration::from_secs(1) {
            pump(&mut emulator, &app, &mut receiver);
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    drop(emulator);
    std::thread::sleep(Duration::from_secs(2));

    status
}
