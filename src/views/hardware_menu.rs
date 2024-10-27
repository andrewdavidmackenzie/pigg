#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::views::connect_dialog_handler::ConnectDialogMessage;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::info_row::MENU_BUTTON_STYLE;
use crate::HardwareTarget::*;
use crate::{Message, ModalMessage};
use iced::widget::{Button, Text};
use iced::{Background, Border, Color, Element, Length, Padding, Renderer, Shadow, Theme};
use iced_aw::menu::{Item, Menu, MenuBar};
use iced_aw::style::menu_bar;

/// Create the view that represents the clickable button that shows what hardware is connected
pub fn view<'a>(
    hardware_view: &'a HardwareView,
    hardware_target: &HardwareTarget,
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
            .style(|_, _| MENU_BUTTON_STYLE),
    );

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let connect: Item<'a, Message, _, _> = Item::new(
        Button::new("Connect to remote Pi ...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(move |_, _| MENU_BUTTON_STYLE),
    );

    #[cfg(not(target_arch = "wasm32"))]
    let connect_local: Item<'a, Message, _, _> = Item::new(
        Button::new("Connect to local")
            .on_press(Message::ConnectRequest(Local))
            .style(move |_, _| MENU_BUTTON_STYLE)
            .width(Length::Fill),
    );

    let show_details = Item::new(
        Button::new(Text::new("Show details..."))
            .on_press(Message::ModalHandle(ModalMessage::HardwareDetailsModal))
            .width(Length::Fill)
            .style(move |_, _| MENU_BUTTON_STYLE),
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
            .width(Length::Fill)
            .style(move |theme, status| MENU_BAR_BUTTON_STYLE),
    ));

    let menu_root = Item::with_menu(
        Button::new(Text::new(model))
            .style(move |_, _| MENU_BUTTON_STYLE)
            .on_press(Message::MenuBarButtonClicked),
        Menu::new(menu_items).width(235.0).offset(10.0),
    );

    // TODO define Style as a const like MENU_BAR_BUTTON_STYLE
    // which is an unused import right now?
    MenuBar::new(vec![menu_root])
        .style(|_, _| menu_bar::Style {
            bar_background: Background::Color(Color::TRANSPARENT),
            bar_border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            bar_shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 0.0,
            },
            bar_background_expand: Padding::new(2.0),
            menu_background: Background::Color(Color::TRANSPARENT),
            menu_border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 2.0.into(),
            },
            menu_shadow: iced::Shadow {
                color: Color::BLACK,
                offset: iced::Vector::new(1.0, 1.0),
                blur_radius: 10f32,
            },
            menu_background_expand: iced::Padding::from([5, 5]),
            path: Background::Color(Color::TRANSPARENT),
            path_border: Default::default(),
        })
        .into()
}
