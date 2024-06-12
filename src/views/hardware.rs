use iced::widget::{container, Column, Text};
use iced::{alignment, Alignment, Element, Length};

use crate::hw::HardwareDescriptor;
use crate::Message;

/// Hardware Configuration Display
pub fn hardware_view(hardware_description: &HardwareDescriptor) -> Element<'static, Message> {
    let hardware_info = Column::new()
        .push(Text::new(format!(
            "Hardware: {}",
            hardware_description.hardware
        )))
        .push(Text::new(format!(
            "Revision: {}",
            hardware_description.revision
        )))
        .push(Text::new(format!(
            "Serial: {}",
            hardware_description.serial
        )))
        .push(Text::new(format!("Model: {}", hardware_description.model)))
        .spacing(10)
        .width(Length::Shrink)
        .align_items(Alignment::Start);

    container(hardware_info)
        .padding(10)
        .width(Length::Shrink)
        .align_x(alignment::Horizontal::Center)
        .into()
}
