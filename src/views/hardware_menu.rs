use crate::hw_definition::description::{HardwareDescription, SsidSpec};
#[cfg(feature = "usb-raw")]
use crate::usb_raw;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog_handler::ConnectDialogMessage;
use crate::views::hardware_menu::KnownDevice::Porky;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::info_row::{
    MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE, MENU_BAR_STYLE, MENU_BUTTON_HOVER_STYLE,
    MENU_BUTTON_STYLE,
};
use crate::HardwareTarget::*;
use crate::{Message, ModalMessage};
use iced::widget::button::Status::Hovered;
use iced::widget::{Button, Text};
use iced::{Element, Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu, MenuBar};
use iced_futures::Subscription;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    USBRaw,
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::USBRaw => f.write_str("on USB"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceFound(DiscoveryMethod, HardwareDescription, Option<SsidSpec>),
    DeviceLost(HardwareDescription),
    Error(String),
}

pub enum KnownDevice {
    Porky(DiscoveryMethod, HardwareDescription, Option<SsidSpec>),
}

/// Create a submenu item for the known devices
fn devices_submenu<'a>(
    known_devices: &HashMap<String, KnownDevice>,
) -> Item<'a, Message, Theme, Renderer> {
    let mut device_items = vec![];

    for (serial_number, device) in known_devices {
        match device {
            Porky(method, hardware_description, _) => {
                let button = Button::new(Text::new(format!(
                    "{} ({}) {}",
                    hardware_description.details.model, serial_number, method
                )))
                .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
                .width(Length::Fill)
                .style(|_, status| {
                    if status == Hovered {
                        MENU_BUTTON_HOVER_STYLE
                    } else {
                        MENU_BUTTON_STYLE
                    }
                });

                if hardware_description.details.wifi {
                    device_items.push(Item::with_menu(
                        button,
                        Menu::new(vec![Item::new(
                            Button::new(Text::new("Configure WiFi"))
                                .on_press(Message::ConfigureWiFi(serial_number.to_string()))
                                .width(150.0)
                                .style(|_, status| {
                                    if status == Hovered {
                                        MENU_BUTTON_HOVER_STYLE
                                    } else {
                                        MENU_BUTTON_STYLE
                                    }
                                }),
                        )])
                        .width(170.0)
                        .offset(10.0),
                    ));
                } else {
                    device_items.push(Item::new(button));
                }
            }
        }
    }

    Item::with_menu(
        Button::new("Discovered devices")
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .width(Length::Fill)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
        Menu::new(device_items).width(270.0).offset(10.0),
    )
}

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view<'a>(
    hardware_view: &'a HardwareView,
    hardware_target: &HardwareTarget,
    known_devices: &HashMap<String, KnownDevice>,
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
        Button::new("Disconnect")
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
        Button::new("Connect to remote Pi ...")
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
        Button::new("Connect to local")
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

    let show_details = Item::new(
        Button::new(Text::new("Show details..."))
            .on_press(Message::ModalHandle(ModalMessage::HardwareDetailsModal))
            .width(Length::Fill)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

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
            menu_items.push(show_details);
        }
        #[cfg(feature = "iroh")]
        Iroh(_, _) => {
            menu_items.push(disconnect);
            menu_items.push(show_details);
        }
        #[cfg(feature = "tcp")]
        Tcp(_, _) => {
            menu_items.push(disconnect);
            menu_items.push(show_details);
        }
    }

    #[cfg(feature = "discovery")]
    menu_items.push(Item::new(
        Button::new("Search for Pi's on local network...")
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

    menu_items.push(devices_submenu(known_devices));

    let hardware_menu_root = Item::with_menu(
        Button::new(Text::new(model))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .style(move |_theme, status| {
                if status == Hovered {
                    MENU_BAR_BUTTON_HOVER_STYLE
                } else {
                    MENU_BAR_BUTTON_STYLE
                }
            }),
        Menu::new(menu_items).width(235.0).offset(10.0),
    );

    MenuBar::new(vec![hardware_menu_root])
        .style(|_, _| MENU_BAR_STYLE)
        .into()
}

/// Create subscriptions for ticks for updating charts of waveforms and events coming from hardware
pub fn subscription() -> Subscription<DeviceEvent> {
    let subscriptions = vec![
        #[cfg(feature = "usb-raw")]
        Subscription::run_with_id("device", usb_raw::subscribe()),
    ];

    Subscription::batch(subscriptions)
}
