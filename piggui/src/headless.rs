//! Headless execution of the Piggui application.
//!
//! [`HeadlessApp`] implements [`iced::Program`] so the real application logic can
//! be driven by `iced_test::Emulator` without opening a host UI window. It runs
//! the same boot, update, view, task and subscription code as the GUI binary,
//! including the real hardware connection path.
//!
//! Use [`HeadlessApp::status`] to observe connection progress from a test.

use crate::{Message, Piggui, StartupConfig};

use iced::window;
use iced::{Element, Pixels, Settings, Subscription, Task, Theme};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// The connection status of a [`HeadlessApp`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Status {
    /// The app has booted but has not reached a terminal connection state yet.
    #[default]
    Starting,
    /// The app connected to hardware.
    Connected,
    /// The app failed to connect to hardware.
    ConnectionError(String),
    /// The app disconnected from hardware.
    Disconnected,
}

/// A cloneable handle used to observe the connection status of a [`HeadlessApp`].
#[derive(Debug, Clone, Default)]
pub struct StatusHook {
    status: Arc<Mutex<Status>>,
    ever_connected: Arc<AtomicBool>,
}

impl StatusHook {
    /// Get the current status.
    #[must_use]
    pub fn get(&self) -> Status {
        self.lock().clone()
    }

    /// Set the current status.
    pub fn set(&self, status: Status) {
        *self.lock() = status;
    }

    /// Record that the app connected to hardware at least once.
    pub fn mark_connected(&self) {
        self.ever_connected.store(true, Ordering::SeqCst);
    }

    /// Whether the app has connected to hardware at least once.
    #[must_use]
    pub fn ever_connected(&self) -> bool {
        self.ever_connected.load(Ordering::SeqCst)
    }

    fn lock(&self) -> MutexGuard<'_, Status> {
        self.status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// A headless Piggui application.
///
/// This can be run by an `iced_test::Emulator` in place of the GUI binary.
#[derive(Debug, Clone)]
pub struct HeadlessApp {
    startup: StartupConfig,
    status: StatusHook,
}

impl HeadlessApp {
    /// Create a headless app that boots with the given [StartupConfig].
    #[must_use]
    pub fn new(startup: StartupConfig) -> Self {
        Self {
            startup,
            status: StatusHook::default(),
        }
    }

    /// Create a headless app that connects to the given hardware on boot.
    #[must_use]
    pub fn with_connection(connection: pignet::HardwareConnection) -> Self {
        Self::new(StartupConfig::new(connection))
    }

    /// Get the current connection status.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status.get()
    }

    /// Whether the app has connected to hardware at least once.
    #[must_use]
    pub fn ever_connected(&self) -> bool {
        self.status.ever_connected()
    }

    /// Get a cloneable handle to observe connection status changes.
    #[must_use]
    pub fn status_hook(&self) -> StatusHook {
        self.status.clone()
    }
}

impl iced::Program for HeadlessApp {
    type State = Piggui;
    type Message = Message;
    type Theme = Theme;
    type Renderer = iced::Renderer;
    type Executor = iced::executor::Default;

    fn name() -> &'static str {
        "piggui"
    }

    fn settings(&self) -> Settings {
        Settings {
            default_text_size: Pixels(14.0),
            ..Settings::default()
        }
    }

    fn window(&self) -> Option<window::Settings> {
        Some(window::Settings::default())
    }

    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        Piggui::with_startup(self.startup.clone())
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        let new_status = match &message {
            Message::Connected => {
                self.status.mark_connected();
                Some(Status::Connected)
            }
            Message::ConnectionError(details) => Some(Status::ConnectionError(details.clone())),
            Message::Disconnect => Some(Status::Disconnected),
            _ => None,
        };

        let task = state.update(message);

        if let Some(status) = new_status {
            self.status.set(status);
        }

        task
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        _window: window::Id,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        state.view()
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        state.subscription()
    }

    fn theme(&self, _state: &Self::State, _window: window::Id) -> Option<Self::Theme> {
        Some(Theme::Dark)
    }
}
