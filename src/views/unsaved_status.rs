use crate::styles::button_style::ButtonStyle;
use crate::Message;
use iced::widget::Button;
use iced::{Color, Length};

use crate::views::info_row::{MENU_BAR_BUTTON_STYLE, MENU_BUTTON_STYLE};
use iced::{Background, Element, Renderer, Theme};
use iced_aw::menu;
use iced_aw::menu::StyleSheet;
use iced_aw::menu::{Item, Menu, MenuBar};
use iced_aw::style::MenuBarStyle;

pub(crate) const MENU_BAR_UNSAVED_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgba(1.0, 0.647, 0.0, 0.7),
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::from_rgba(1.0, 0.647, 0.0, 1.0),
    border_radius: 2.0,
};

/// Create the view that represents the status of unsaved changes in the info row
pub fn view<'a>(unsaved_changes: bool) -> Element<'a, Message, Theme, Renderer> {
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let load_from: Item<'a, Message, _, _> = Item::new(
        Button::new("Load config from...")
            .width(Length::Fill)
            .on_press(Message::Load)
            .style(MENU_BUTTON_STYLE.get_button_style()),
    );

    menu_items.push(load_from);

    let save_as: Item<'a, Message, _, _> = Item::new(
        Button::new("Save config as...")
            .width(Length::Fill)
            .on_press(Message::Save)
            .style(MENU_BUTTON_STYLE.get_button_style()),
    );

    if unsaved_changes {
        menu_items.push(save_as);
    }

    let button = match unsaved_changes {
        true => Button::new("config: unsaved changes")
            .style(MENU_BAR_UNSAVED_BUTTON_STYLE.get_button_style()),
        false => Button::new("config").style(MENU_BAR_BUTTON_STYLE.get_button_style()),
    }
    .on_press(Message::MenuBarButtonClicked);

    let menu_root = Item::with_menu(button, Menu::new(menu_items).width(135.0).offset(10.0));

    MenuBar::new(vec![menu_root])
        .style(|theme: &iced::Theme| menu::Appearance {
            bar_background: Background::Color(Color::TRANSPARENT),
            menu_shadow: iced::Shadow {
                color: Color::BLACK,
                offset: iced::Vector::new(1.0, 1.0),
                blur_radius: 10f32,
            },
            menu_background_expand: iced::Padding::from([5, 5]),
            ..theme.appearance(&MenuBarStyle::Default)
        })
        .into()
}
