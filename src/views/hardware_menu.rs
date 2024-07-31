use iced::widget::{Button, Text};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};

use crate::connect_dialog_handler::ConnectDialogMessage;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::info_row::{MENU_BAR_BUTTON_STYLE, MENU_BUTTON_STYLE};
use crate::{Message, ToastMessage};

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn item<'a>(
    hardware_view: &'a HardwareView,
    hardware_target: &HardwareTarget,
) -> Item<'a, Message, Theme, Renderer> {
    let model = match hardware_view.hw_model() {
        None => "No Hardware connected".to_string(),
        Some(model) => match hardware_target {
            HardwareTarget::Local => format!("{}@Local", model),
            HardwareTarget::Remote(_, _) => format!("{}@Remote", model),
            HardwareTarget::NoHW => "No Hardware connected".to_string(),
        },
    };

    // Conditionally render menu items based on hardware features
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    #[cfg(any(feature = "pi_hw", feature = "fake_hw"))]
    menu_items.push(Item::new(
        Button::new("Use local GPIO")
            .on_press(Message::ConnectDialog(ConnectDialogMessage::ConnectLocal))
            .style(MENU_BUTTON_STYLE.get_button_style())
            .width(Length::Fill),
    ));

    menu_items.push(Item::new(
        Button::new("Connect to remote Pi...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(MENU_BUTTON_STYLE.get_button_style()),
    ));

    #[cfg(feature = "discovery")]
    menu_items.push(Item::new(
        Button::new("Search for Pi's on local network...")
            .width(Length::Fill)
            .style(MENU_BUTTON_STYLE.get_button_style()),
    ));

    menu_items.push(Item::new(
        Button::new(Text::new("Show Hardware Details"))
            .on_press(Message::Toast(ToastMessage::HardwareDetailsToast))
            .width(Length::Fill)
            .style(MENU_BUTTON_STYLE.get_button_style()),
    ));

    Item::with_menu(
        Button::new(Text::new(model))
            .style(MENU_BAR_BUTTON_STYLE.get_button_style())
            .on_press(Message::MenuBarButtonClicked),
        Menu::new(menu_items).width(200.0).spacing(2.0).offset(10.0),
    )
}
