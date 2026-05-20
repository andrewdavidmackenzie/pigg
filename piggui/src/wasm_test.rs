use super::*;
use crate::hardware_subscription::SubscriptionEvent;
use crate::views::hardware_view::HardwareViewMessage::SubscriptionMessage;
use pigdef::config::HardwareConfig;
use pignet::HardwareConnection::NoConnection;
use std::collections::HashMap;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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

#[wasm_bindgen_test]
fn app_initializes_in_wasm() {
    let app = test_piggui();
    assert!(app.hardware_view.get_description().is_none());
    assert!(!app.unsaved_changes);
}

#[wasm_bindgen_test]
fn app_connects_to_fake_hardware_in_wasm() {
    let app = test_piggui_connected();
    assert!(app.hardware_view.get_description().is_some());
}

#[wasm_bindgen_test]
fn disconnected_view_renders_in_wasm() {
    let app = test_piggui();
    let _view = app.view();
}

#[wasm_bindgen_test]
fn connected_view_renders_in_wasm() {
    let app = test_piggui_connected();
    let _view = app.view();
}
