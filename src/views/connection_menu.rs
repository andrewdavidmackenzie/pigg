#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog::ConnectDialogMessage;
use crate::views::hardware_view::HardwareView;
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{menu_bar_button, menu_button};
use crate::HardwareConnection::*;
use crate::Message;
use iced::widget::{button, text};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};
use std::collections::HashMap;

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view<'a>(hardware_view: &'a HardwareView) -> Item<'a, Message, Theme, Renderer> {
    // Conditionally render menu items based on hardware features
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let disconnect: Item<'a, Message, _, _> = Item::new(
        button("Disconnect")
            .width(Length::Fill)
            .on_press(Message::Disconnect)
            .style(menu_button),
    );

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let connect: Item<'a, Message, _, _> = Item::new(
        button("Connect to remote Pi ...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(menu_button),
    );

    if let Some(hardware_description) = hardware_view.hardware_description.as_ref() {
        let show_details = Item::new(
            button("Display Device Details...")
                .on_press(Message::Modal(HardwareDetailsModal(
                    hardware_description.details.clone(),
                    HashMap::default(),
                )))
                .width(Length::Fill)
                .style(menu_button),
        );
        menu_items.push(show_details);
    }

    match hardware_view.get_hardware_connection() {
        NoConnection => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
        }
        Local => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            menu_items.push(disconnect);
        }
        #[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
        _ => {
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            menu_items.push(disconnect);
        }
    }

    let model_string = format!(
        "{}: {}",
        hardware_view.get_hardware_connection().name(),
        hardware_view.hw_model().unwrap_or(""),
    );
    Item::with_menu(
        button(text(model_string))
            .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
            .style(menu_bar_button),
        Menu::new(menu_items).width(215.0).offset(10.0),
    )
}
