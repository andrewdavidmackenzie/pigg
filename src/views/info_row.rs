use iced::widget::{container, Row};
use iced::{Color, Command, Element, Length};
use iced_futures::Subscription;

use crate::styles::background::SetAppearance;
use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::layout_selector::LayoutSelector;
use crate::views::message_row::{MessageMessage, MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{hardware_menu, unsaved_status};
use crate::Message;

pub(crate) const MENU_BAR_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgba(0.7, 0.7, 0.7, 1.0),
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::WHITE,
    border_radius: 2.0,
};

pub(crate) const MENU_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::WHITE,
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::WHITE,
    border_radius: 4.0,
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
    pub fn update(&mut self, message: MessageRowMessage) -> Command<Message> {
        self.message_row.update(message)
    }

    /// Create the view that represents the info row at the bottom of the window
    pub fn view<'a>(
        &'a self,
        unsaved_changes: bool,
        layout_selector: &'a LayoutSelector,
        hardware_view: &'a HardwareView,
        hardware_target: &'a HardwareTarget,
    ) -> Element<'a, Message> {
        container(
            Row::new()
                .push(version_button())
                .push(layout_selector.view(hardware_target))
                .push(hardware_menu::view(hardware_view, hardware_target))
                .push(unsaved_status::view(unsaved_changes))
                .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
                .push(self.message_row.view().map(Message::InfoRow))
                .spacing(20.0)
                .padding([0.0, 0.0, 0.0, 0.0]),
        )
        .set_background(Color::from_rgb8(40, 40, 40))
        .into()
    }

    pub fn subscription(&self) -> Subscription<MessageRowMessage> {
        self.message_row.subscription()
    }
}
