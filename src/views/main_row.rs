use crate::views::configuration_column;
use crate::views::hardware_view::HardwareView;
use crate::views::layout_selector::LayoutSelector;
use crate::Message;
use iced::widget::{container, Column, Row};
use iced::{Alignment, Element, Length};

/// Construct the view that represents the main row of the app
pub fn view<'a>(
    hardware_view: &'a HardwareView,
    layout_selector: &'a LayoutSelector,
) -> Element<'a, Message> {
    let mut main_row = Row::new();

    main_row = main_row.push(
        Column::new()
            .push(configuration_column::view(layout_selector))
            .align_items(Alignment::Start)
            .width(Length::Shrink)
            .height(Length::Shrink),
    );

    main_row = main_row.push(
        Column::new()
            .push(
                hardware_view
                    .view(layout_selector.get())
                    .map(Message::Hardware),
            )
            .align_items(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Fill),
    );

    container(main_row).padding([10.0, 10.0, 0.0, 10.0]).into()
}
