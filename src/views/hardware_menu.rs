#[cfg(feature = "usb-raw")]
use crate::hw_definition::description::HardwareDescription;
#[cfg(feature = "usb-raw")]
use crate::hw_definition::description::WiFiDetails;
#[cfg(feature = "usb-raw")]
use crate::usb_raw;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog::ConnectDialogMessage;
#[cfg(feature = "usb-raw")]
use crate::views::hardware_menu::KnownDevice::Porky;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{
    MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE, MENU_BAR_STYLE, MENU_BUTTON_HOVER_STYLE,
    MENU_BUTTON_STYLE,
};
#[cfg(feature = "usb-raw")]
use crate::views::ssid_dialog::SsidDialogMessage;
use crate::HardwareTarget::*;
use crate::Message;
#[cfg(feature = "usb-raw")]
use iced::alignment;
use iced::widget::button::Status::Hovered;
use iced::widget::{button, text};
#[cfg(feature = "usb-raw")]
use iced::widget::{horizontal_space, row};
use iced::{Element, Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu, MenuBar};
#[cfg(feature = "usb-raw")]
use iced_futures::Subscription;
#[cfg(feature = "usb-raw")]
use std::collections::HashMap;
#[cfg(feature = "usb-raw")]
use std::fmt::{Display, Formatter};

#[cfg(feature = "usb-raw")]
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    USBRaw,
}

#[cfg(feature = "usb-raw")]
impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::USBRaw => f.write_str("on USB"),
        }
    }
}

#[cfg(feature = "usb-raw")]
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceFound(DiscoveryMethod, HardwareDescription, Option<WiFiDetails>),
    DeviceLost(HardwareDescription),
    Error(String),
}

#[cfg(feature = "usb-raw")]
pub enum KnownDevice {
    Porky(DiscoveryMethod, HardwareDescription, Option<WiFiDetails>),
}

#[cfg(feature = "usb-raw")]
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
                    device_items.push(Item::with_menu(
                        device_button,
                        Menu::new(vec![
                            Item::new(
                                button("Configure Device Wi-Fi...")
                                    .width(Length::Fill)
                                    .on_press(Message::SsidDialog(
                                        SsidDialogMessage::ShowSsidDialog(
                                            hardware_description.details.clone(),
                                            wifi_details.as_ref().unwrap().ssid_spec.clone(), // TODO unwrap
                                        ),
                                    ))
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
                        ])
                        .width(280.0)
                        .offset(10.0),
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
    hardware_target: &HardwareTarget,
    #[cfg(feature = "usb-raw")] known_devices: &HashMap<String, KnownDevice>,
) -> Element<'a, Message, Theme, Renderer> {
    let model = match hardware_view.hw_model() {
        None => "hardware: none".to_string(),
        Some(model) => match hardware_target {
            NoHW => "hardware: none".to_string(),
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
            .on_press(Message::ConnectRequest(NoHW))
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

    match hardware_target {
        NoHW => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            #[cfg(not(target_arch = "wasm32"))]
            menu_items.push(connect_local);
        }
        #[cfg(not(target_arch = "wasm32"))]
        Local => {
            menu_items.push(disconnect);
        }
        #[cfg(feature = "iroh")]
        Iroh(_, _) => {
            menu_items.push(disconnect);
        }
        #[cfg(feature = "tcp")]
        Tcp(_, _) => {
            menu_items.push(disconnect);
        }
    }

    #[cfg(feature = "discovery")]
    menu_items.push(Item::new(
        button("Search for Pi's on local network...")
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .width(Length::Fill)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    ));

    #[cfg(feature = "usb-raw")]
    menu_items.push(devices_submenu(known_devices));

    let hardware_menu_root = Item::with_menu(
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
    );

    MenuBar::new(vec![hardware_menu_root])
        .style(|_, _| MENU_BAR_STYLE)
        .into()
}

#[cfg(feature = "usb-raw")]
/// Create subscriptions for ticks for updating charts of waveforms and events coming from hardware
pub fn subscription() -> Subscription<DeviceEvent> {
    let subscriptions = vec![
        #[cfg(feature = "usb-raw")]
        Subscription::run_with_id("device", usb_raw::subscribe()),
    ];

    Subscription::batch(subscriptions)
}
