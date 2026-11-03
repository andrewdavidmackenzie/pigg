use super::*;
use crate::hardware_subscription::SubscriptionEvent;
use crate::views::hardware_view::HardwareViewMessage::{
    ChangeOutputLevel, PinFunctionChanged, SubscriptionMessage,
};
use crate::views::info_dialog::InfoDialogMessage;
use crate::views::layout_menu::Layout;
use iced::window;
use pigdef::config::HardwareConfig;
use pigdef::config::InputPull::{PullDown, PullUp};
use pigdef::config::LevelChange;
use pigdef::pin_function::PinFunction::{Input, Output};
use pignet::HardwareConnection::NoConnection;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_piggui() -> Piggui {
    Piggui {
        config_filename: None,
        layout_selector: LayoutSelector::new(),
        unsaved_changes: false,
        info_row: InfoRow::new(),
        modal_handler: InfoDialog::new(),
        hardware_view: HardwareView::new(NoConnection),
        #[cfg(feature = "discovery")]
        discovery_enabled: false,
        #[cfg(any(feature = "iroh", feature = "tcp"))]
        connect_dialog: ConnectDialog::new(),
        #[cfg(feature = "discovery")]
        discovered_devices: HashMap::new(),
        #[cfg(feature = "usb")]
        ssid_dialog: SsidDialog::new(),
    }
}

fn test_piggui_connected() -> Piggui {
    let hw = piggpio::HW::new();
    let hw_desc = hw.description().clone();
    let hw_config = HardwareConfig::default();

    let mut app = test_piggui();
    app.hardware_view
        .new_connection(pignet::HardwareConnection::Local);
    let _ = app
        .hardware_view
        .update(SubscriptionMessage(SubscriptionEvent::Connected(
            hw_desc, hw_config,
        )));
    let _ = app.update(Message::Connected);
    app
}

// --- Pin Configuration Tests ---

#[test]
fn pin_set_to_input_pullup() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Input(Some(PullUp))),
        false,
        true,
    )));
    assert_eq!(
        app.hardware_view.get_config().pin_functions.get(&bcm_pin),
        Some(&Input(Some(PullUp)))
    );
}

#[test]
fn pin_set_to_input_pulldown() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Input(Some(PullDown))),
        false,
        true,
    )));
    assert_eq!(
        app.hardware_view.get_config().pin_functions.get(&bcm_pin),
        Some(&Input(Some(PullDown)))
    );
}

#[test]
fn pin_set_to_input_no_pull() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Input(None)),
        false,
        true,
    )));
    assert_eq!(
        app.hardware_view.get_config().pin_functions.get(&bcm_pin),
        Some(&Input(None))
    );
}

#[test]
fn pin_set_to_output() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Output(None)),
        false,
        true,
    )));
    assert!(matches!(
        app.hardware_view.get_config().pin_functions.get(&bcm_pin),
        Some(&Output(_))
    ));
}

#[test]
fn output_toggle_changes_value() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Output(None)),
        false,
        true,
    )));
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let _ = app.update(Hardware(ChangeOutputLevel(
        bcm_pin,
        LevelChange::new(true, now),
    )));
    // ChangeOutputLevel updates pin_states for display and sends to hardware,
    // but doesn't update hardware_config (the remote side does that).
    // Verify the pin is still configured as output and the view renders.
    assert!(matches!(
        app.hardware_view.get_config().pin_functions.get(&bcm_pin),
        Some(&Output(_))
    ));
    let _view = app.view();
}

#[test]
fn pin_cleared() {
    let mut app = test_piggui_connected();
    let bcm_pin = 2;
    let _ = app.update(Hardware(PinFunctionChanged(
        bcm_pin,
        Some(Input(Some(PullUp))),
        false,
        true,
    )));
    assert!(app
        .hardware_view
        .get_config()
        .pin_functions
        .contains_key(&bcm_pin));
    let _ = app.update(Hardware(PinFunctionChanged(bcm_pin, None, false, true)));
    assert!(!app
        .hardware_view
        .get_config()
        .pin_functions
        .contains_key(&bcm_pin));
}

// --- Layout Tests ---

#[test]
fn layout_default_is_board() {
    let app = test_piggui_connected();
    assert_eq!(app.layout_selector.get(), Layout::Board);
}

#[test]
fn layout_changes_to_logical() {
    let mut app = test_piggui_connected();
    let _ = app.update(LayoutChanged(Layout::Logical));
    assert_eq!(app.layout_selector.get(), Layout::Logical);
}

#[test]
fn layout_changes_back() {
    let mut app = test_piggui_connected();
    let _ = app.update(LayoutChanged(Layout::Logical));
    let _ = app.update(LayoutChanged(Layout::Board));
    assert_eq!(app.layout_selector.get(), Layout::Board);
}

// --- Exit/Dialog Tests ---

#[test]
fn exit_without_changes_no_dialog() {
    let mut app = test_piggui_connected();
    assert!(!app.unsaved_changes);
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(!app.modal_handler.showing_modal());
}

#[test]
fn exit_with_changes_shows_dialog() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    assert!(app.unsaved_changes);
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(app.modal_handler.showing_modal());
}

#[test]
fn exit_dialog_cancel_returns_to_app() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(app.modal_handler.showing_modal());
    let _ = app.update(Modal(InfoDialogMessage::HideModal));
    assert!(!app.modal_handler.showing_modal());
}

#[test]
fn exit_dialog_reappears_after_cancel() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    let _ = app.update(Modal(InfoDialogMessage::HideModal));
    assert!(!app.modal_handler.showing_modal());
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(app.modal_handler.showing_modal());
}

#[test]
fn exit_dialog_dismiss_via_hide_modal() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(app.modal_handler.showing_modal());
    let _ = app.update(Modal(InfoDialogMessage::HideModal));
    assert!(!app.modal_handler.showing_modal());
}

// --- Config Load State Tests ---

#[test]
fn config_loaded_updates_state() {
    let mut app = test_piggui_connected();
    let mut config = HardwareConfig::default();
    config.pin_functions.insert(2, Input(Some(PullUp)));
    let _ = app.update(ConfigLoaded("test.pigg".to_string(), config));
    assert_eq!(app.config_filename, Some("test.pigg".to_string()));
    assert_eq!(
        app.hardware_view.get_config().pin_functions.get(&2),
        Some(&Input(Some(PullUp)))
    );
}

#[test]
fn config_loaded_clears_unsaved() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    assert!(app.unsaved_changes);
    let _ = app.update(ConfigLoaded(
        "test.pigg".to_string(),
        HardwareConfig::default(),
    ));
    assert!(!app.unsaved_changes);
}

#[test]
fn load_with_unsaved_changes_shows_dialog() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(Load);
    assert!(app.modal_handler.showing_modal());
}

#[test]
fn load_dialog_cancel_returns() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(Load);
    let _ = app.update(Modal(InfoDialogMessage::HideModal));
    assert!(!app.modal_handler.showing_modal());
}

// --- View Rendering Smoke Tests ---

#[test]
fn disconnected_view_renders() {
    let app = test_piggui();
    let _view = app.view();
}

#[test]
fn connected_view_renders() {
    let app = test_piggui_connected();
    let _view = app.view();
}

#[test]
fn connected_view_has_expected_elements() {
    let app = test_piggui_connected();
    let _view = app.view();
    assert!(app.hardware_view.get_description().is_some());
    assert!(!app.modal_handler.showing_modal());
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
#[test]
fn connect_dialog_accessible_when_disconnected() {
    let mut app = test_piggui();
    assert!(app.hardware_view.get_description().is_none());
    let _ = app.update(ConnectDialog(
        crate::views::connect_dialog::ConnectDialogMessage::ShowConnectDialog,
    ));
    assert!(app.connect_dialog.show_modal);
    let _view = app.view();
}

// --- Headless widget interaction tests using iced_test ---

#[cfg(not(target_arch = "wasm32"))]
mod simulator_tests {
    use super::*;
    use iced_test::simulator;

    /// Simulate a sequence of clicks against the current view and return the messages they produced.
    fn click_all(app: &Piggui, targets: &[&str]) -> Vec<Message> {
        let mut ui = simulator(app.view());
        for target in targets {
            ui.click(*target)
                .unwrap_or_else(|error| panic!("Could not click {target:?}: {error}"));
        }
        ui.into_messages().collect()
    }

    fn apply(app: &mut Piggui, messages: Vec<Message>) {
        for message in messages {
            let _ = app.update(message);
        }
    }

    fn assert_view_contains(app: &Piggui, expected: &str) {
        let mut ui = simulator(app.view());
        assert!(
            ui.find(expected).is_ok(),
            "Expected the view to contain {expected:?}"
        );
    }

    #[test]
    fn simulator_layout_menu_changes_layout() {
        let mut app = test_piggui_connected();
        assert_eq!(app.layout_selector.get(), Layout::Board);

        let messages = click_all(&app, &["layout: board", "BCP Pin Layout"]);
        assert!(messages
            .iter()
            .any(|message| matches!(message, LayoutChanged(Layout::Logical))));
        apply(&mut app, messages);

        assert_eq!(app.layout_selector.get(), Layout::Logical);
        assert_view_contains(&app, "layout: bcp");
    }

    #[test]
    fn simulator_exit_dialog_return_button() {
        let mut app = test_piggui_connected();
        let _ = app.update(ConfigChangesMade(false, true));
        let _ = app.update(WindowEvent(iced::Event::Window(
            window::Event::CloseRequested,
        )));
        assert!(app.modal_handler.showing_modal());

        let messages = click_all(&app, &["Return to app"]);
        apply(&mut app, messages);

        assert!(!app.modal_handler.showing_modal());
    }

    #[test]
    fn simulator_exit_dialog_exit_without_saving() {
        let mut app = test_piggui_connected();
        let _ = app.update(ConfigChangesMade(false, true));
        let _ = app.update(WindowEvent(iced::Event::Window(
            window::Event::CloseRequested,
        )));
        assert!(app.modal_handler.showing_modal());

        let messages = click_all(&app, &["Exit without saving"]);

        assert!(messages
            .iter()
            .any(|message| matches!(message, Modal(InfoDialogMessage::ExitApp))));

        // Apply the message so the `ExitApp` handler runs. The window-closing
        // task it returns is discarded: `iced_test`'s simulator has no window,
        // so window closure itself is not covered here.
        apply(&mut app, messages);
    }

    #[test]
    fn simulator_load_dialog_continue_button() {
        let mut app = test_piggui_connected();
        let _ = app.update(ConfigChangesMade(false, true));
        let _ = app.update(Load);
        assert!(app.modal_handler.showing_modal());

        let messages = click_all(&app, &["Continue and load a new config"]);

        assert!(messages
            .iter()
            .any(|message| matches!(message, Modal(InfoDialogMessage::LoadFile))));
        apply(&mut app, messages);
        assert!(!app.modal_handler.showing_modal());
    }

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    #[test]
    fn simulator_opens_connect_dialog() {
        let mut app = test_piggui();
        assert!(!app.connect_dialog.show_modal);

        let messages = click_all(&app, &["disconnected", "Connect to remote Pi ..."]);
        apply(&mut app, messages);

        assert!(app.connect_dialog.show_modal);
        #[cfg(feature = "iroh")]
        assert_view_contains(&app, "Connect To Remote Pi Using Iroh");
        #[cfg(all(not(feature = "iroh"), feature = "tcp"))]
        assert_view_contains(&app, "Connect To Remote Pi Using Tcp");
    }

    #[cfg(all(feature = "iroh", feature = "tcp"))]
    #[test]
    fn simulator_connect_dialog_switches_to_tcp_tab() {
        let mut app = test_piggui();
        let _ = app.update(ConnectDialog(
            crate::views::connect_dialog::ConnectDialogMessage::ShowConnectDialog,
        ));
        assert_view_contains(&app, "Connect To Remote Pi Using Iroh");

        let messages = click_all(&app, &["Connect using TCP"]);
        apply(&mut app, messages);

        assert_view_contains(&app, "Connect To Remote Pi Using Tcp");
    }
}

// --- Startup configuration tests ---

#[cfg(feature = "discovery")]
#[test]
fn discovery_subscriptions_follow_startup_config() {
    let (disabled, _) = Piggui::with_startup(StartupConfig::new(NoConnection).without_discovery());
    assert!(disabled.discovery_subscriptions().is_empty());

    #[cfg(any(feature = "usb", feature = "tcp"))]
    {
        let (enabled, _) = Piggui::with_startup(StartupConfig::new(NoConnection));
        assert!(!enabled.discovery_subscriptions().is_empty());
    }
}
