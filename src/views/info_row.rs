use iced::widget::{container, Row};
use iced::Subscription;
use iced::{Color, Element, Length, Task};
use iced_aw::menu::MenuBar;

use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::message_row::{MessageMessage, MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{hardware_menu, unsaved_status};
use crate::Message;


// TODO
// TODO hovered_bg_color: Color::TRANSPARENT,
// TODO hovered_text_color: Color::WHITE,


// TODO
// TODO hovered_bg_color: Color::TRANSPARENT,
// TODO hovered_text_color: Color::WHITE,

#[derive(Default)]
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
        hardware_view: &'a HardwareView,
        hardware_target: &HardwareTarget,
    ) -> Element<'a, Message> {
        let hardware_root = hardware_menu::item(hardware_view, hardware_target);

        let mb = MenuBar::new(vec![hardware_root]);

        container(
            Row::new()
                .push(version_button())
                .push(mb)
                .push(unsaved_status::view(unsaved_changes))
                .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
                .push(self.message_row.view().map(Message::InfoRow))
                .spacing(20.0)
                .padding(0),
        )
        .set_background(Color::from_rgb8(45, 45, 45))
        .into()
    }

    pub fn subscription(&self) -> Subscription<MessageRowMessage> {
        self.message_row.subscription()
    }
}
