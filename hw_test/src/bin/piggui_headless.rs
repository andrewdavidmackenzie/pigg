//! A short-lived, headless `piggui` process used by the hardware tests.
//!
//! The real `piggui` application is driven with `iced_test::Emulator`, so it
//! exercises the real boot/update/view/subscription code and the real hardware
//! connection path without opening a host UI window.
//!
//! The process prints a single `HEADLESS_STATUS:` line once it reaches a
//! terminal state and then exits. Running each connection in its own process is
//! what guarantees that USB interfaces, Iroh endpoints and TCP sockets are
//! released: the operating system closes them when the process exits.

use std::net::IpAddr;
use std::process::ExitCode;
use std::str::FromStr;
use std::time::{Duration, Instant};

use iced_test::core::Size;
use iced_test::emulator::{self, Emulator};
use iced_test::futures::futures::channel::mpsc;
use piggui::headless::{HeadlessApp, Status};
use piggui::StartupConfig;
use pignet::HardwareConnection;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (startup, timeout) = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(error) => {
            println!("HEADLESS_STATUS: CONNECTION_ERROR {error}");
            return ExitCode::FAILURE;
        }
    };

    let app = HeadlessApp::new(startup);
    let status = run_until_terminal(&app, timeout);

    match status {
        Status::Connected => {
            println!("HEADLESS_STATUS: CONNECTED");
            ExitCode::SUCCESS
        }
        Status::ConnectionError(details) => {
            println!("HEADLESS_STATUS: CONNECTION_ERROR {details}");
            ExitCode::FAILURE
        }
        Status::Disconnected => {
            println!("HEADLESS_STATUS: DISCONNECTED");
            ExitCode::FAILURE
        }
        Status::Starting => {
            println!("HEADLESS_STATUS: TIMEOUT");
            ExitCode::FAILURE
        }
    }
}

/// Run the real application with `iced_test::Emulator` until it connects, fails,
/// or times out.
///
/// A successful connection is reported as soon as it has been observed, even if
/// the app subsequently reports a non-fatal hardware event error. This matches
/// what the GUI shows the user and what the tests assert.
fn run_until_terminal(app: &HeadlessApp, timeout: Duration) -> Status {
    let (sender, mut receiver) = mpsc::channel(256);
    let mut emulator = Emulator::new(sender, app, emulator::Mode::Zen, Size::new(1024.0, 768.0));

    let start = Instant::now();

    loop {
        pump(&mut emulator, app, &mut receiver);

        if app.ever_connected() {
            // Ask the app to close the connection cleanly before this process
            // exits, so devices are not left in a bad state for the next test.
            disconnect(&mut emulator, app, &mut receiver);
            return Status::Connected;
        }

        let status = app.status();
        if matches!(status, Status::ConnectionError(_)) || start.elapsed() > timeout {
            return status;
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Drive the emulator until its event queue is empty.
fn pump(
    emulator: &mut Emulator<HeadlessApp>,
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

/// Send a disconnect and give the subscription time to close the connection.
fn disconnect(
    emulator: &mut Emulator<HeadlessApp>,
    app: &HeadlessApp,
    receiver: &mut mpsc::Receiver<emulator::Event<HeadlessApp>>,
) {
    emulator.update(app, piggui::Message::Disconnect);

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        pump(emulator, app, receiver);
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn parse_args(args: &[String]) -> Result<(StartupConfig, Duration), String> {
    let mut connection = HardwareConnection::NoConnection;
    let mut timeout = DEFAULT_TIMEOUT;

    #[cfg(feature = "iroh")]
    let mut endpoint_id = None;
    #[cfg(feature = "iroh")]
    let mut relay = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--usb" => {
                let serial = value(args, &mut index, "--usb")?;
                #[cfg(feature = "usb")]
                {
                    connection = HardwareConnection::Usb(serial.to_string());
                }
                #[cfg(not(feature = "usb"))]
                {
                    let _ = serial;
                    return Err("USB support is not enabled in this build".to_string());
                }
            }
            "--ip" => {
                let address = value(args, &mut index, "--ip")?;
                #[cfg(feature = "tcp")]
                {
                    let (ip, port) = address
                        .split_once(':')
                        .ok_or_else(|| format!("Expected 'IP:port', got '{address}'"))?;
                    let ip = IpAddr::from_str(ip)
                        .map_err(|e| format!("Invalid IP address '{ip}': {e}"))?;
                    let port =
                        u16::from_str(port).map_err(|e| format!("Invalid port '{port}': {e}"))?;
                    connection = HardwareConnection::Tcp(ip, port);
                }
                #[cfg(not(feature = "tcp"))]
                {
                    let _ = address;
                    return Err("TCP support is not enabled in this build".to_string());
                }
            }
            "--endpoint-id" => {
                let value = value(args, &mut index, "--endpoint-id")?;
                #[cfg(feature = "iroh")]
                {
                    endpoint_id = Some(
                        iroh::EndpointId::from_str(value)
                            .map_err(|e| format!("Invalid endpoint id '{value}': {e}"))?,
                    );
                }
                #[cfg(not(feature = "iroh"))]
                {
                    let _ = value;
                    return Err("Iroh support is not enabled in this build".to_string());
                }
            }
            "--relay" => {
                let value = value(args, &mut index, "--relay")?;
                #[cfg(feature = "iroh")]
                {
                    relay = Some(
                        iroh::RelayUrl::from_str(value)
                            .map_err(|e| format!("Invalid relay URL '{value}': {e}"))?,
                    );
                }
                #[cfg(not(feature = "iroh"))]
                {
                    let _ = value;
                    return Err("Iroh support is not enabled in this build".to_string());
                }
            }
            "--timeout" => {
                let seconds = value(args, &mut index, "--timeout")?;
                timeout = Duration::from_secs(
                    u64::from_str(seconds)
                        .map_err(|e| format!("Invalid timeout '{seconds}': {e}"))?,
                );
            }
            unknown => return Err(format!("Unknown argument '{unknown}'")),
        }

        index += 1;
    }

    #[cfg(feature = "iroh")]
    if let Some(endpoint_id) = endpoint_id {
        connection = HardwareConnection::Iroh(endpoint_id, relay);
    } else if relay.is_some() {
        return Err("'--relay' requires '--endpoint-id'".to_string());
    }

    Ok((StartupConfig::new(connection).without_discovery(), timeout))
}

fn value<'a>(args: &'a [String], index: &mut usize, flag: &str) -> Result<&'a str, String> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| format!("Missing value for '{flag}'"))
}
