#[cfg(feature = "discovery")]
use crate::discovery::DiscoveredDevice;
use crate::views::hardware_view::HardwareView;
use crate::views::layout_menu::LayoutSelector;
use crate::views::message_box::{MessageMessage, MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{config_menu, connection_menu, devices_menu};
use crate::Message;
use iced::border::Radius;
use iced::widget::{button, container, Row};
use iced::{Background, Border, Color, Element, Length, Padding, Shadow, Task};
use iced_aw::style::menu_bar;
use iced_aw::MenuBar;
use iced_futures::Subscription;
#[cfg(feature = "discovery")]
use std::collections::HashMap;

const MENU_BACKGROUND_COLOR: Color = Color::from_rgba(0.15, 0.15, 0.15, 1.0);

const MENU_RADIUS: Radius = Radius {
    top_left: 4.0,
    top_right: 4.0,
    bottom_right: 4.0,
    bottom_left: 4.0,
};

const BLACK_SHADOW: Shadow = Shadow {
    color: Color::BLACK,
    offset: iced::Vector::new(1.0, 1.0),
    blur_radius: 5f32,
};

const HOVERED_COLOR: Color = Color::WHITE;
const UNHOVERED_COLOR: Color = Color::from_rgba(0.7, 0.7, 0.7, 1.0);

const MENU_BORDER: Border = Border {
    color: Color::TRANSPARENT,
    width: 0.0,
    radius: MENU_RADIUS,
};

const MENU_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: iced::Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

pub(crate) const MENU_BAR_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: UNHOVERED_COLOR,
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub(crate) const MENU_BAR_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: HOVERED_COLOR,
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub(crate) const MENU_BAR_BUTTON_HIGHLIGHT_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: Color::from_rgba(1.0, 0.647, 0.0, 0.7),
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub(crate) const MENU_BUTTON_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: UNHOVERED_COLOR,
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub(crate) const MENU_BUTTON_HOVER_STYLE: button::Style = button::Style {
    background: Some(Background::Color(Color::TRANSPARENT)),
    text_color: HOVERED_COLOR,
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub(crate) const MENU_BAR_STYLE: menu_bar::Style = menu_bar::Style {
    bar_background: Background::Color(Color::TRANSPARENT),
    bar_border: MENU_BORDER,
    bar_shadow: MENU_SHADOW,
    bar_background_expand: Padding::new(2.0),
    menu_background: Background::Color(MENU_BACKGROUND_COLOR),
    menu_border: MENU_BORDER,
    menu_shadow: BLACK_SHADOW,
    menu_background_expand: Padding::new(5.0),
    path: Background::Color(Color::TRANSPARENT),
    path_border: MENU_BORDER,
};

const INFO_BAR_STYLE: container::Style = container::Style {
    text_color: Some(Color::WHITE),
    background: Some(Background::Color(MENU_BACKGROUND_COLOR)),
    border: MENU_BORDER,
    shadow: MENU_SHADOW,
};

pub struct InfoRow {
    message_row: MessageRow,
}

impl InfoRow {
    /// Create a new InfoRow
    pub fn new() -> Self {
        Self {
            message_row: MessageRow::new(),
        }
    }

    /// Add a message to the queue of messages to display in the message_row
    pub fn add_info_message(&mut self, msg: MessageMessage) {
        self.message_row.add_message(msg);
    }

    /// Update state based on [MessageRowMessage] messages received
    pub fn update(&mut self, message: MessageRowMessage) -> Task<Message> {
        self.message_row.update(message)
    }

    /// Create the view that represents the info row at the bottom of the window
    pub fn view<'a>(
        &'a self,
        unsaved_changes: bool,
        layout_selector: &'a LayoutSelector,
        hardware_view: &'a HardwareView,
        #[cfg(feature = "discovery")] discovered_devices: &HashMap<String, DiscoveredDevice>,
    ) -> Element<'a, Message> {
        let menu_bar: Element<Message> = MenuBar::new(vec![
            version_button(),
            layout_selector.view(hardware_view.get_hardware_connection()),
            connection_menu::view(hardware_view),
            #[cfg(feature = "discovery")]
            devices_menu::view(hardware_view.get_hardware_connection(), discovered_devices),
            config_menu::view(unsaved_changes, hardware_view.get_hardware_connection()),
        ])
        .style(|_, _| MENU_BAR_STYLE)
        .into();

        container(
            Row::new()
                .push(menu_bar)
                .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
                .push(self.message_row.view().map(Message::InfoRow))
                .spacing(20.0)
                .padding(Padding::new(0.0)),
        )
        .style(|_theme| INFO_BAR_STYLE)
        .into()
    }

    pub fn subscription(&self) -> Subscription<MessageRowMessage> {
        self.message_row.subscription()
    }
}
