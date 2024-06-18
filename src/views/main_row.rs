use crate::pin_layout::{bcm_pin_layout_view, board_pin_layout_view};
use crate::views::configuration_column;
use crate::views::layout_selector::Layout;
use crate::{Message, Piggui};
use iced::widget::{Column, container, Row};
use iced::{Alignment, Element, Length};

/// Construct the view that represents the main row of the app
pub fn view(app: &Piggui) -> Element<Message> {
    let mut main_row = Row::new();

    main_row = main_row.push(
        Column::new()
            .push(configuration_column::view(app))
            .align_items(Alignment::Start)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(app.layout_selector.get_spacing()),
    );

    main_row = main_row.push(
            Column::new()
                .push(app.hardware_view.view(app, app.layout_selector.get()))
                .align_items(Alignment::Center)
                .height(Length::Fill)
                .width(Length::Fill),
        );

    container(main_row).padding([10.0, 10.0, 0.0, 10.0]).into()
}
