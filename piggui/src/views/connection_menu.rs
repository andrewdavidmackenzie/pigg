use crate::views::hardware_view::HardwareView;
use crate::views::info_dialog::InfoDialogMessage::HardwareDetailsModal;
use crate::views::info_row::{menu_bar_button, menu_button_style};
use crate::Message;
use iced::widget::{button, text};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};
use pignet::HardwareConnection::NoConnection;
use std::collections::HashMap;

/// Create the menu for actions related to connected hardware
pub fn view<'a>(hardware_view: &'a HardwareView) -> Item<'a, Message, Theme, Renderer> {
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let connection_string = if hardware_view.get_hardware_connection() == &NoConnection {
        "disconnected".to_string()
    } else if let Some(hardware_description) = hardware_view.get_description() {
        let show_details = Item::new(
            button("Display Device Details...")
                .on_press(Message::Modal(HardwareDetailsModal(
                    hardware_description.details.clone(),
                    HashMap::default(),
                )))
                .width(Length::Fill)
                .style(menu_button_style),
        );
        menu_items.push(show_details);

        #[cfg(any(feature = "iroh", feature = "tcp", not(target_arch = "wasm32")))]
        let disconnect: Item<'a, Message, _, _> = Item::<Message, Theme, Renderer>::new(
            button("Disconnect")
                .width(Length::Fill)
                .on_press(Message::Disconnect)
                .style(menu_button_style),
        );
        #[cfg(any(feature = "iroh", feature = "tcp", not(target_arch = "wasm32")))]
        menu_items.push(disconnect);

        format!(
            "{}: {}",
            hardware_view.get_hardware_connection().name(),
            hardware_description.details.model.as_str()
        )
    } else {
        "connecting...".to_string()
    };

    if menu_items.is_empty() {
        Item::new(button(text(connection_string)).style(menu_bar_button))
    } else {
        Item::with_menu(
            button(text(connection_string))
                .on_press(Message::MenuBarButtonClicked) // Needed for highlighting
                .style(menu_bar_button),
            Menu::new(menu_items).width(215.0),
        )
    }
}
