use iced::{alignment, Alignment, Element, Length};
use iced::widget::{Column, container, Text};

use crate::hw::HardwareDescriptor;
use crate::Message;

/// Hardware Configuration Display
pub fn hardware_view(hardware_description: &HardwareDescriptor) -> Element<'static, Message> {
    let hardware_info = Column::new()
        .push(Text::new(format!("Hardware: {}", hardware_description.hardware)).size(20))
        .push(Text::new(format!("Revision: {}", hardware_description.revision)).size(20))
        .push(Text::new(format!("Serial: {}", hardware_description.serial)).size(20))
        .push(Text::new(format!("Model: {}", hardware_description.model)).size(20))
        .spacing(10)
        .width(Length::Fill)
        .align_items(Alignment::Start);

    container(hardware_info)
        .padding(10)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
}
