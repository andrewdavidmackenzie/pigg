use crate::Message;
use iced::widget::Button;

use crate::views::hardware_view::HardwareConnection;
use crate::views::hardware_view::HardwareConnection::NoConnection;
use crate::views::info_row::{
    MENU_BAR_BUTTON_HIGHLIGHT_STYLE, MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE,
    MENU_BUTTON_HOVER_STYLE, MENU_BUTTON_STYLE,
};
use iced::widget::button::Status::Hovered;
use iced::{Renderer, Theme};
use iced_aw::menu::{Item, Menu};

/// Create the view that represents the status of unsaved changes in the info row
pub fn view<'a>(
    unsaved_changes: bool,
    hardware_connection: &HardwareConnection,
) -> Item<'a, Message, Theme, Renderer> {
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let mut load_from = Button::new("Load config from...")
        .width(180)
        .style(|_, status| {
            if status == Hovered {
                MENU_BUTTON_HOVER_STYLE
            } else {
                MENU_BUTTON_STYLE
            }
        });

    if hardware_connection != &NoConnection {
        load_from = load_from.on_press(Message::Load);
    }
    menu_items.push(Item::new(load_from));

    if unsaved_changes {
        let mut save_as = Button::new("Save config as...")
            .width(180)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            });

        if hardware_connection != &NoConnection {
            save_as = save_as.on_press(Message::Save);
        }

        menu_items.push(Item::new(save_as));
    }

    let mut button = match unsaved_changes {
        true => Button::new("config: unsaved changes").style(|_, status| {
            if status == Hovered {
                MENU_BAR_BUTTON_HOVER_STYLE
            } else {
                MENU_BAR_BUTTON_HIGHLIGHT_STYLE
            }
        }),
        false => Button::new("config").style(move |_theme, status| {
            if status == Hovered {
                MENU_BAR_BUTTON_HOVER_STYLE
            } else {
                MENU_BAR_BUTTON_STYLE
            }
        }),
    };
    button = button.on_press(Message::MenuBarButtonClicked); // Needed for highlighting

    // Increased width to 145 as Linux needs a little more width
    Item::with_menu(button, Menu::new(menu_items).width(180.0).offset(10.0))
}
