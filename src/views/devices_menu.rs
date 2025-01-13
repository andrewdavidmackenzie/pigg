use crate::discovery::DiscoveredDevice;
#[cfg(feature = "usb")]
use crate::discovery::DiscoveryMethod::USBRaw;
use crate::views::hardware_view::HardwareConnection;
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{MENU_BUTTON_HOVER_STYLE, MENU_BUTTON_STYLE};
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialogMessage;
use crate::HardwareConnection::*;
use crate::Message;
use iced::alignment;
use iced::widget::button::Status::Hovered;
use iced::widget::{button, text};
use iced::widget::{horizontal_space, row};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};
use std::collections::HashMap;

/// Create a submenu item for the known devices
#[cfg(feature = "discovery")]
fn device_items<'a>(
    discovered_devices: &HashMap<String, DiscoveredDevice>,
    current_connection: &HardwareConnection,
) -> Vec<Item<'a, Message, Theme, Renderer>> {
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

        // TODO avoid the current connected device altogether
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
                .style(|_, status| {
                    if status == Hovered {
                        MENU_BUTTON_HOVER_STYLE
                    } else {
                        MENU_BUTTON_STYLE
                    }
                }),
        ));

        // Add buttons to connect to the device for each available connection type, except
        // for a [HardwareConnection] type currently used to connect to the device
        for hardware_connection in hardware_connections.values() {
            if !matches!(hardware_connection, NoConnection) {
                // avoid re-offering the current connection method if connected
                let connect_button = if current_connection != hardware_connection {
                    button(text(format!("Connect via {}", hardware_connection.name())))
                        .on_press(Message::ConnectRequest(hardware_connection.clone()))
                        .width(Length::Fill)
                        .style(|_, status| {
                            if status == Hovered {
                                MENU_BUTTON_HOVER_STYLE
                            } else {
                                MENU_BUTTON_STYLE
                            }
                        })
                } else {
                    button(text("Connected to Device"))
                        .width(Length::Fill)
                        .style(|_, status| {
                            if status == Hovered {
                                MENU_BUTTON_HOVER_STYLE
                            } else {
                                MENU_BUTTON_STYLE
                            }
                        })
                };
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
                        .style(|_, status| {
                            if status == Hovered {
                                MENU_BUTTON_HOVER_STYLE
                            } else {
                                MENU_BUTTON_STYLE
                            }
                        }),
                ));
            }

            if matches!(discovery_method, USBRaw) {
                device_menu_items.push(Item::new(
                    button("Reset Device Wi-Fi to Default")
                        .width(Length::Fill)
                        .on_press(Message::ResetSsid(hardware_details.serial.clone()))
                        .style(|_, status| {
                            if status == Hovered {
                                MENU_BUTTON_HOVER_STYLE
                            } else {
                                MENU_BUTTON_STYLE
                            }
                        }),
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
        .style(|_: &Theme, status| {
            if status == Hovered {
                MENU_BUTTON_HOVER_STYLE
            } else {
                MENU_BUTTON_STYLE
            }
        });

        device_items.push(Item::with_menu(
            device_button,
            Menu::new(device_menu_items).width(280.0).offset(10.0),
        ));
    }

    device_items
}

/// Create the discovered devices menu with items for each discovered device
pub fn view<'a>(
    hardware_connection: &HardwareConnection,
    #[cfg(feature = "discovery")] discovered_devices: &HashMap<String, DiscoveredDevice>,
) -> Item<'a, Message, Theme, Renderer> {
    let device_items = device_items(discovered_devices, hardware_connection);

    Item::with_menu(
        button(text(format!("devices ({})", device_items.len())))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
        Menu::new(device_items).width(Length::Shrink).offset(10.0),
    )
}
