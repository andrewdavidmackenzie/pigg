use crate::views::layout_selector::LayoutSelector;
use crate::Message;
use iced::widget::Column;
use iced::{Alignment, Element, Length};

/// Construct the view that represents the configuration column
pub fn view(layout_selector: &LayoutSelector) -> Element<'static, Message> {
    let mut configuration_column = Column::new()
        .align_items(Alignment::Start)
        .spacing(10)
        .width(Length::Shrink);
    configuration_column = configuration_column.push(layout_selector.view());

    configuration_column.into()
}
