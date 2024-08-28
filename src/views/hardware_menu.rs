use iced::widget::{Button, Text};
use iced::{Length, Renderer, Theme};
use iced_aw::menu::{Item, Menu};

use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::info_row::{MENU_BAR_BUTTON_STYLE, MENU_BUTTON_STYLE};
use crate::HardwareTarget::*;
use crate::{Message, ModalMessage};

#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog_handler::ConnectDialogMessage;

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn item<'a>(
    hardware_view: &'a HardwareView,
    hardware_target: &HardwareTarget,
) -> Item<'a, Message, Theme, Renderer> {
    let model = match hardware_view.hw_model() {
        None => "No Hardware connected".to_string(),
        Some(model) => match hardware_target {
            NoHW => "No Hardware connected".to_string(),
            Local => format!("{}@Local", model),
            #[cfg(feature = "iroh")]
            Iroh(_, _) => format!("{}@Remote", model),
            #[cfg(feature = "tcp")]
            Tcp(_, _) => format!("{}@Remote", model),
        },
    };

    // Conditionally render menu items based on hardware features
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let disconnect: Item<'a, Message, _, _> = Item::new(
        Button::new("Disconnect from Hardware")
            .width(Length::Fill)
            .on_press(Message::ConnectRequest(NoHW))
            .style(MENU_BUTTON_STYLE.get_button_style()),
    );

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let connect: Item<'a, Message, _, _> = Item::new(
        Button::new("Connect to remote Pi ...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(MENU_BUTTON_STYLE.get_button_style()),
    );

    #[cfg(not(target_arch = "wasm32"))]
    let connect_local: Item<'a, Message, _, _> = Item::new(
        Button::new("Connect to local Hardware")
            .on_press(Message::ConnectRequest(Local))
            .style(MENU_BUTTON_STYLE.get_button_style())
            .width(Length::Fill),
    );

    let show_details = Item::new(
        Button::new(Text::new("Show Hardware Details..."))
            .on_press(Message::ModalHandle(ModalMessage::HardwareDetailsModal))
            .width(Length::Fill)
            .style(MENU_BUTTON_STYLE.get_button_style()),
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
            #[cfg(any(feature = "iroh", feature = "tcp"))]
            menu_items.push(connect);
            menu_items.push(show_details);
        }
        #[cfg(feature = "iroh")]
        Iroh(_, _) => {
            menu_items.push(disconnect);
            #[cfg(not(target_arch = "wasm32"))]
            menu_items.push(connect_local);
            menu_items.push(show_details);
        }
        #[cfg(feature = "tcp")]
        Tcp(_, _) => {
            menu_items.push(disconnect);
            #[cfg(not(target_arch = "wasm32"))]
            menu_items.push(connect_local);
            menu_items.push(show_details);
        }
    }

    #[cfg(feature = "discovery")]
    menu_items.push(Item::new(
        Button::new("Search for Pi's on local network...")
            .width(Length::Fill)
            .style(MENU_BUTTON_STYLE.get_button_style()),
    ));

    Item::with_menu(
        Button::new(Text::new(model))
            .style(MENU_BAR_BUTTON_STYLE.get_button_style())
            .on_press(Message::MenuBarButtonClicked),
        Menu::new(menu_items).width(235.0).offset(10.0),
    )
}
