use iced::widget::{Button, Text};
use iced::{Background, Border, Color, Length, Renderer, Theme};
use iced::border::Radius;
use iced_aw::menu::{Item, Menu};

use crate::connect_dialog_handler::ConnectDialogMessage;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::HardwareTarget::NoHW;
use crate::HardwareTarget::Remote;
use crate::{Message, ModalMessage};
use iced::widget::button::Style;
/// Create the view that represents the clickable button that shows what hardware is connected
pub fn item<'a>(
    hardware_view: &'a HardwareView,
    hardware_target: &HardwareTarget,
) -> Item<'a, Message, Theme, Renderer> {
    let menu_button_style: Style = Style {
            background: Some(Background::from(Color::TRANSPARENT)),
            text_color: Color::WHITE,
            border: Border {
                radius: Radius::from(4),
                ..Default::default()
            },
            shadow: Default::default(),
        };
    let model = match hardware_view.hw_model() {
        None => "No Hardware connected".to_string(),
        Some(model) => match hardware_target {
            NoHW => "No Hardware connected".to_string(),
            HardwareTarget::Local => format!("{}@Local", model),
            Remote(_, _) => format!("{}@Remote", model),
        },
    };

    // Conditionally render menu items based on hardware features
    let mut menu_items: Vec<Item<'a, Message, _, _>> = vec![];

    let disconnect: Item<'a, Message, _, _> = Item::new(
        Button::new("Disconnect from Hardware")
            .width(Length::Fill)
            .on_press(Message::ConnectRequest(NoHW))
            .style(move |_theme, _status| menu_button_style),
    );

    let connect_remote: Item<'a, Message, _, _> = Item::new(
        Button::new("Connect to remote Pi...")
            .width(Length::Fill)
            .on_press(Message::ConnectDialog(
                ConnectDialogMessage::ShowConnectDialog,
            ))
            .style(move |_theme, _status| menu_button_style),
    );

    let connect_local: Item<'a, Message, _, _> = Item::new(
        Button::new("Use local GPIO")
            .on_press(Message::ConnectRequest(HardwareTarget::Local))
            .style(move |_theme, _status| menu_button_style)
            .width(Length::Fill),
    );

    match hardware_target {
        NoHW => {
            menu_items.push(connect_remote);
            menu_items.push(connect_local);
        }
        HardwareTarget::Local => {
            menu_items.push(disconnect);
            menu_items.push(connect_remote);
        }
        Remote(_, _) => {
            menu_items.push(disconnect);
            menu_items.push(connect_local);
        }
    }

    menu_items.push(Item::new(
        Button::new("Search for Pi's on local network...")
            .width(Length::Fill)
            .style(move |_theme, _status| menu_button_style),
    ));

    menu_items.push(Item::new(
        Button::new(Text::new("Show Hardware Details..."))
            .on_press(Message::ModalHandle(ModalMessage::HardwareDetailsModal))
            .width(Length::Fill)
            .style(move |_theme, _status| menu_button_style),
    ));

    Item::with_menu(
        Button::new(Text::new(model))
            .style(move |__theme, _status| menu_button_style)
            .on_press(Message::MenuBarButtonClicked),
        Menu::new(menu_items).width(200.0).spacing(2.0).offset(10.0),
    )
}
