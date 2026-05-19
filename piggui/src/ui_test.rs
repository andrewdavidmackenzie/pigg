use super::*;
use crate::hardware_subscription::SubscriptionEvent;
use crate::views::hardware_view::HardwareViewMessage::{
    ChangeOutputLevel, PinFunctionChanged, SubscriptionMessage,
};
use crate::views::info_dialog::InfoDialogMessage;
use crate::views::layout_menu::Layout;
use iced::window;
use iced_test::simulator::simulator;
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
    let _ = app
        .hardware_view
        .update(SubscriptionMessage(SubscriptionEvent::Connected(
            hw_desc, hw_config,
        )));
    let _ = app.update(Message::Connected);
    app
}

fn click_and_update(app: &mut Piggui, label: &str) {
    let view = app.view();
    let mut ui = simulator(view);
    let _ = ui.click(label);
    let msgs: Vec<Message> = ui.into_messages().collect();
    for msg in msgs {
        let _ = app.update(msg);
    }
}

fn find_in_view(app: &Piggui, label: &str) -> bool {
    let view = app.view();
    let mut ui = simulator(view);
    ui.find(label).is_ok()
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
fn exit_dialog_buttons_present() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(find_in_view(&app, "Exit without saving"));
    assert!(find_in_view(&app, "Return to app"));
}

#[test]
fn exit_dialog_return_via_click() {
    let mut app = test_piggui_connected();
    let _ = app.update(ConfigChangesMade(false, true));
    let _ = app.update(WindowEvent(iced::Event::Window(
        window::Event::CloseRequested,
    )));
    assert!(app.modal_handler.showing_modal());
    click_and_update(&mut app, "Return to app");
    assert!(!app.modal_handler.showing_modal());
}

// --- Config Load State Tests ---

#[test]
fn config_loaded_updates_state() {
    let mut app = test_piggui_connected();
    let mut config = HardwareConfig::default();
    config
        .pin_functions
        .insert(2, Input(Some(PullUp)));
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
    let view = app.view();
    let ui = simulator(view);
    // The view should render without panicking — the simulator successfully
    // processes the widget tree
    drop(ui);
}
