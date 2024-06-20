use crate::styles::background::SetAppearance;
use crate::views::hardware_view::HardwareView;
use crate::views::message_row::MessageRow;
use crate::views::version::version_button;
use crate::views::{hardware_button, unsaved_status};
use crate::Message;
use iced::widget::{container, Row};
use iced::{Color, Element, Length};

/// Create the view that represents the info row at the bottom of the window
pub fn view<'a>(
    unsaved_changes: bool,
    hardware_view: &'a HardwareView,
    status_row: &'a MessageRow,
) -> Element<'a, Message> {
    container(
        Row::new()
            .push(version_button())
            .push(hardware_button::view(hardware_view))
            .push(unsaved_status::view(unsaved_changes))
            .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
            .push(status_row.view().map(Message::StatusRow))
            .spacing(20.0)
            .padding([0.0, 0.0, 0.0, 0.0]),
    )
    .set_background(Color::from_rgb8(45, 45, 45))
    .into()
}
