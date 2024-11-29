use crate::discovery::KnownDevice;
#[cfg(feature = "usb")]
use crate::hw_definition::description::WiFiDetails;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog::ConnectDialogMessage;
#[cfg(feature = "usb")]
use crate::views::hardware_menu::KnownDevice::Porky;
use crate::views::hardware_view::{HardwareConnection, HardwareView};
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{
    MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE, MENU_BUTTON_HOVER_STYLE, MENU_BUTTON_STYLE,
};
#[cfg(feature = "usb")]
use crate::views::ssid_dialog::SsidDialogMessage;
use crate::HardwareConnection::*;
use crate::Message;
#[cfg(feature = "usb")]
use iced::alignment;
use iced::widget::button::Status::Hovered;
use iced::widget::{button, text};
#[cfg(feature = "usb")]
use iced::widget::{horizontal_space, row};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};
#[cfg(feature = "usb")]
use std::collections::HashMap;
#[cfg(all(feature = "tcp", feature = "usb"))]
use std::net::{IpAddr, Ipv4Addr};

#[cfg(feature = "discovery")]
/// Create a submenu item for the known devices
fn devices_submenu<'a>(
    known_devices: &HashMap<String, KnownDevice>,
) -> Item<'a, Message, Theme, Renderer> {
    let mut device_items = vec![];

    for (serial_number, device) in known_devices {
        match device {
            Porky(method, hardware_description, wifi_details) => {
                let device_button = button(row!(
                    text(format!(
                        "{} ({}) {}",
                        hardware_description.details.model, serial_number, method
                    )),
                    horizontal_space(),
                    text(" >").align_y(alignment::Vertical::Center),
                ))
                .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
                .style(|_, status| {
                    if status == Hovered {
                        MENU_BUTTON_HOVER_STYLE
                    } else {
                        MENU_BUTTON_STYLE
                    }
                });

                if hardware_description.details.wifi {
                    #[allow(unused_mut)]
                    let mut menu_items = vec![
                        Item::new(
                            button("Configure Device Wi-Fi...")
                                .width(Length::Fill)
                                .on_press(Message::SsidDialog(SsidDialogMessage::Show(
                                    hardware_description.details.clone(),
                                    wifi_details.as_ref().and_then(|wf| wf.ssid_spec.clone()),
                                )))
                                .style(|_, status| {
                                    if status == Hovered {
                                        MENU_BUTTON_HOVER_STYLE
                                    } else {
                                        MENU_BUTTON_STYLE
                                    }
                                }),
                        ),
                        Item::new(
                            button("Display Device Details...")
                                .width(Length::Fill)
                                .on_press(Message::Modal(HardwareDetailsModal(
                                    hardware_description.details.clone(),
                                    wifi_details.as_ref().unwrap().tcp,
                                )))
                                .style(|_, status| {
                                    if status == Hovered {
                                        MENU_BUTTON_HOVER_STYLE
                                    } else {
                                        MENU_BUTTON_STYLE
                                    }
                                }),
                        ),
                        Item::new(
                            button("Reset Device Wi-Fi to Default")
                                .width(Length::Fill)
                                .on_press(Message::ResetSsid(
                                    hardware_description.details.serial.clone(),
                                ))
                                .style(|_, status| {
                                    if status == Hovered {
                                        MENU_BUTTON_HOVER_STYLE
                                    } else {
                                        MENU_BUTTON_STYLE
                                    }
                                }),
                        ),
                    ];

                    #[cfg(feature = "tcp")]
                    if let Porky(
                        _,
                        _,
                        Some(WiFiDetails {
                            ssid_spec: _,
                            tcp: Some((ip, port)),
                        }),
                    ) = device
                    {
                        let target =
                            Tcp(IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])), *port);
                        menu_items.push(Item::new(
                            button("Connect to Device by TCP")
                                .width(Length::Fill)
                                .on_press(Message::ConnectRequest(target))
                                .style(|_, status| {
                                    if status == Hovered {
                                        MENU_BUTTON_HOVER_STYLE
                                    } else {
                                        MENU_BUTTON_STYLE
                                    }
                                }),
                        ));
                    }

                    device_items.push(Item::with_menu(
                        device_button,
                        Menu::new(menu_items).width(280.0).offset(10.0),
                    ));
                } else {
                    device_items.push(Item::new(device_button));
                }
            }
        }
    }

    if device_items.is_empty() {
        Item::new(
            button("Discovered devices (None)")
                .width(Length::Fill)
                .style(|_, status| {
                    if status == Hovered {
                        MENU_BUTTON_HOVER_STYLE
                    } else {
                        MENU_BUTTON_STYLE
                    }
                }),
        )
    } else {
        Item::with_menu(
            button(row!(
                text(format!("Discovered devices ({})", device_items.len())),
                horizontal_space(),
                text(" >").align_y(alignment::Vertical::Center)
            ))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .width(Length::Fill)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
            Menu::new(device_items).width(280.0).offset(10.0),
        )
    }
}

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view<'a>(
    hardware_view: &'a HardwareView,
    hardware_connection: &HardwareConnection,
    #[cfg(feature = "usb")] known_devices: &HashMap<String, KnownDevice>,
) -> Item<'a, Message, Theme, Renderer> {
    let model = match hardware_view.hw_model() {
        None => "hardware: none".to_string(),
        Some(model) => match hardware_connection {
            NoConnection => "hardware: none".to_string(),
            Local => format!("hardware: {}@Local", model),
            #[cfg(feature = "iroh")]
            Iroh(_, _) => format!("hardware: {}@Remote", model),
            #[cfg(feature = "tcp")]
            Tcp(_, _) => format!("hardware: {}@Remote", model),
        },
    };

    // Conditionally render menu items based on hardware features
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let disconnect: Item<'a, Message, _, _> = Item::new(
        button("Disconnect")
            .width(Length::Fill)
            .on_press(Message::Disconnected)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let connect: Item<'a, Message, _, _> = Item::new(
        button("Connect to remote Pi ...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

    #[cfg(not(target_arch = "wasm32"))]
    let connect_local: Item<'a, Message, _, _> = Item::new(
        button("Connect to local")
            .width(Length::Fill)
            .on_press(Message::ConnectRequest(Local))
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

    if let Some(hardware_description) = hardware_view.hardware_description.as_ref() {
        let show_details = Item::new(
            button("Show details...")
                .on_press(Message::Modal(HardwareDetailsModal(
                    hardware_description.details.clone(),
                    None,
                )))
                .width(Length::Fill)
                .style(|_, status| {
                    if status == Hovered {
                        MENU_BUTTON_HOVER_STYLE
                    } else {
                        MENU_BUTTON_STYLE
                    }
                }),
        );
        menu_items.push(show_details);
    }

    match hardware_connection {
        NoConnection => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            #[cfg(not(target_arch = "wasm32"))]
            menu_items.push(connect_local);
        }
        #[cfg(not(target_arch = "wasm32"))]
        Local => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            menu_items.push(disconnect);
        }
        #[cfg(any(feature = "iroh", feature = "tcp"))]
        _ => {
            menu_items.push(connect);
            menu_items.push(disconnect);
            #[cfg(not(target_arch = "wasm32"))]
            menu_items.push(connect_local);
        }
    }

    #[cfg(feature = "discovery")]
    menu_items.push(devices_submenu(known_devices));

    Item::with_menu(
        button(text(model))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .style(move |_theme, status| {
                if status == Hovered {
                    MENU_BAR_BUTTON_HOVER_STYLE
                } else {
                    MENU_BAR_BUTTON_STYLE
                }
            }),
        Menu::new(menu_items).width(215.0).offset(10.0),
    )
}
