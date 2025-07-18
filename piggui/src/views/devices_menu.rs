#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog::ConnectDialogMessage;
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{menu_bar_button, menu_button_style};
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialogMessage;
use crate::Message;
use iced::alignment;
use iced::widget::{button, text};
use iced::widget::{horizontal_space, row};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};
use pignet::discovery::DiscoveredDevice;
#[cfg(feature = "usb")]
use pignet::discovery::DiscoveryMethod::USBRaw;
use pignet::HardwareConnection;
use pignet::HardwareConnection::*;
use std::collections::HashMap;

/// Create a submenu item for the known devices
#[cfg(feature = "discovery")]
fn device_menu_items<'a>(
    discovered_devices: &HashMap<String, DiscoveredDevice>,
    current_connection: &HardwareConnection,
) -> (Vec<Item<'a, Message, Theme, Renderer>>, usize) {
    #[allow(unused_mut)]
    let mut device_items = vec![];

    #[allow(unused_variables)]
    for (
        key,
        DiscoveredDevice {
            discovery_method,
            hardware_details,
            ssid_spec,
            hardware_connections,
        },
    ) in discovered_devices
    {
        // Menu items under each device menu
        let mut device_menu_items: Vec<Item<Message, Theme, Renderer>> = vec![];

        // Avoid the current connection being a connect option in the details dialog
        let mut connect_options = hardware_connections.clone();
        connect_options.remove(current_connection.name());
        device_menu_items.push(Item::new(
            button("Display Device Details...")
                .width(Length::Fill)
                .on_press(Message::Modal(HardwareDetailsModal(
                    hardware_details.clone(),
                    connect_options,
                )))
                .style(menu_button_style),
        ));

        // Add buttons to connect to the device for each available connection type, except
        // for a [HardwareConnection] type currently used to connect to the device
        for hardware_connection in hardware_connections.values() {
            if !matches!(hardware_connection, NoConnection) {
                let mut connect_button =
                    button(text(format!("Connect via {}", hardware_connection.name())))
                        .width(Length::Fill)
                        .style(menu_button_style);

                // avoid re-offering the current connection method if connected
                if current_connection != hardware_connection {
                    connect_button = connect_button
                        .on_press(Message::ConnectRequest(hardware_connection.clone()));
                }
                device_menu_items.push(Item::new(connect_button));
            }
        }

        // Section for menu items to allow config of Wi-Fi of a Pico W via USB
        #[cfg(feature = "usb")]
        if hardware_details.wifi {
            if matches!(discovery_method, USBRaw) {
                #[allow(unused_mut)]
                device_menu_items.push(Item::new(
                    button("Configure Device Wi-Fi...")
                        .width(Length::Fill)
                        .on_press(Message::SsidDialog(SsidDialogMessage::Show(
                            hardware_details.clone(),
                            ssid_spec.as_ref().and_then(|_wf| ssid_spec.clone()),
                        )))
                        .style(menu_button_style),
                ));
            }

            if matches!(discovery_method, USBRaw) {
                device_menu_items.push(Item::new(
                    button("Reset Device Wi-Fi to Default")
                        .width(Length::Fill)
                        .on_press(Message::ResetSsid(hardware_details.serial.clone()))
                        .style(menu_button_style),
                ));
            }
        }

        // Button for each device menu
        let device_button = button(row!(
            text(format!("{} ({})", hardware_details.model, key)),
            horizontal_space(),
            text(" >").align_y(alignment::Vertical::Center),
        ))
        .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
        .style(menu_button_style);

        device_items.push(Item::with_menu(
            device_button,
            Menu::new(device_menu_items).width(200.0),
        ));
    }

    let device_count = device_items.len();

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    device_items.push(Item::new(
        button("Connect to remote Pi ...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(menu_button_style),
    ));

    (device_items, device_count)
}

/// Create the discovered devices menu with items for each discovered device
pub fn view<'a>(
    hardware_connection: &HardwareConnection,
    #[cfg(feature = "discovery")] discovered_devices: &HashMap<String, DiscoveredDevice>,
) -> Item<'a, Message, Theme, Renderer> {
    let (device_menu_items, device_count) =
        device_menu_items(discovered_devices, hardware_connection);

    Item::with_menu(
        button(text(format!("devices ({device_count})")))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .style(menu_bar_button),
        Menu::new(device_menu_items).width(380.0).max_width(400.0),
    )
}
