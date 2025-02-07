use crate::Message;
use iced::widget::Button;

use crate::views::info_row::{menu_bar_button, menu_bar_highlight_button, menu_button_style};
use iced::{Renderer, Theme};
use iced_aw::menu::{Item, Menu};
use pignet::HardwareConnection;
use pignet::HardwareConnection::NoConnection;

/// Create the view that represents the status of unsaved changes in the info row
pub fn view<'a>(
    unsaved_changes: bool,
    hardware_connection: &HardwareConnection,
) -> Item<'a, Message, Theme, Renderer> {
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let mut load_from = Button::new("Load config from...")
        .width(180)
        .style(menu_button_style);

    if hardware_connection != &NoConnection {
        load_from = load_from.on_press(Message::Load);
    }
    menu_items.push(Item::new(load_from));

    if unsaved_changes {
        let mut save_as = Button::new("Save config as...")
            .width(180)
            .style(menu_button_style);

        if hardware_connection != &NoConnection {
            save_as = save_as.on_press(Message::Save);
        }

        menu_items.push(Item::new(save_as));
    }

    let mut button = match unsaved_changes {
        true => Button::new("config: unsaved changes").style(menu_bar_highlight_button),
        false => Button::new("config").style(menu_bar_button),
    };
    button = button.on_press(Message::MenuBarButtonClicked); // Needed for highlighting

    // Increased width to 145 as Linux needs a little more width
    Item::with_menu(button, Menu::new(menu_items).width(180.0))
}
