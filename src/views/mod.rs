use crate::views::info_row::{
    MENU_BAR_BUTTON_HIGHLIGHT_STYLE, MENU_BAR_BUTTON_HOVER_STYLE, MENU_BAR_BUTTON_STYLE,
    MENU_BUTTON_HOVER_STYLE, MENU_BUTTON_STYLE,
};
use iced::widget::button::Status::Hovered;
use iced::widget::button::{Status, Style};
use iced::Theme;

pub mod about;
pub mod config_menu;
#[cfg(any(feature = "iroh", feature = "tcp"))]
pub mod connect_dialog;
pub mod connection_menu;
#[cfg(feature = "discovery")]
pub mod devices_menu;
mod dialog_styles;
mod hardware_styles;
pub mod hardware_view;
pub mod info_dialog;
pub mod info_row;
pub mod layout_menu;
pub mod message_box;
pub mod pin_state;
#[cfg(feature = "usb")]
pub mod ssid_dialog;
pub mod waveform;

pub fn menu_button(_: &Theme, status: Status) -> Style {
    if status == Hovered {
        MENU_BUTTON_HOVER_STYLE
    } else {
        MENU_BUTTON_STYLE
    }
}

pub fn menu_bar_button(_: &Theme, status: Status) -> Style {
    if status == Hovered {
        MENU_BAR_BUTTON_HOVER_STYLE
    } else {
        MENU_BAR_BUTTON_STYLE
    }
}

pub fn menu_bar_highlight_button(_: &Theme, status: Status) -> Style {
    if status == Hovered {
        MENU_BAR_BUTTON_HOVER_STYLE
    } else {
        MENU_BAR_BUTTON_HIGHLIGHT_STYLE
    }
}
