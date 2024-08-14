use crate::views::configuration_column;
use crate::views::hardware_view::{HardwareTarget, HardwareView};
use crate::views::layout_selector::LayoutSelector;
use crate::Message;
use iced::widget::{container, Column, Row};
use iced::{Alignment, Element, Length};

/// Construct the view that represents the main row of the app
pub fn view<'a>(
    hardware_view: &'a HardwareView,
    layout_selector: &'a LayoutSelector,
    hardware_target: &'a HardwareTarget,
) -> Element<'a, Message> {
    let mut main_row = Row::new();

    main_row = main_row.push(
        Column::new()
            .push(configuration_column::view(layout_selector))
            .align_x(Alignment::Start)
            .width(Length::Shrink)
            .height(Length::Shrink),
    );

    main_row = main_row.push(
        Column::new()
            .push(
                hardware_view
                    .view(layout_selector.get(), hardware_target)
                    .map(Message::Hardware),
            )
            .align_x(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill),
    );

    container(main_row).padding([10.0, 10.0, 0.0, 10.0]).into()
}
