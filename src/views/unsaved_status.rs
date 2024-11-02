use crate::Message;
use iced::widget::Button;
use iced::Length;

use crate::views::info_row::{
    MENU_BAR_BUTTON_HIGHLIGHT_STYLE, MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE,
    MENU_BAR_STYLE, MENU_BUTTON_HOVER_STYLE, MENU_BUTTON_STYLE,
};
use iced::widget::button::Status::Hovered;
use iced::{Element, Renderer, Theme};
use iced_aw::menu::{Item, Menu, MenuBar};

/// Create the view that represents the status of unsaved changes in the info row
pub fn view<'a>(unsaved_changes: bool) -> Element<'a, Message, Theme, Renderer> {
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let load_from: Item<'a, Message, _, _> = Item::new(
        Button::new("Load config from...")
            .width(Length::Fill)
            .on_press(Message::Load)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

    menu_items.push(load_from);

    let save_as: Item<'a, Message, _, _> = Item::new(
        Button::new("Save config as...")
            .width(Length::Fill)
            .on_press(Message::Save)
            .style(|_, status| {
                if status == Hovered {
                    MENU_BUTTON_HOVER_STYLE
                } else {
                    MENU_BUTTON_STYLE
                }
            }),
    );

    if unsaved_changes {
        menu_items.push(save_as);
    }

    let button = match unsaved_changes {
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

    // Increased width to 145 as Linux needs a little more width
    let menu_root = Item::with_menu(button, Menu::new(menu_items).width(145.0).offset(10.0));

    MenuBar::new(vec![menu_root])
        .style(|_, _| MENU_BAR_STYLE)
        .into()
}
