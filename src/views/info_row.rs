use crate::styles::background::SetAppearance;
use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::HardwareView;
use crate::views::message_row::{MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{hardware_button, unsaved_status};
use crate::Message;
use iced::widget::{container, Button, Column, Row, Text};
use iced::{Color, Command, Element, Length};
use iced_aw::menu;
use iced_aw::menu::{Item, Menu, MenuBar};
use iced_aw::menu_bar;
use iced_futures::Subscription;

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

    /// Update state based on [MessageRowMessage] messages received
    pub fn update(&mut self, message: MessageRowMessage) -> Command<Message> {
        self.message_row.update(message)
    }

    /// Create the view that represents the info row at the bottom of the window
    pub fn view<'a>(
        &'a self,
        unsaved_changes: bool,
        hardware_view: &'a HardwareView,
    ) -> Element<'a, Message> {
        let about_button_style = ButtonStyle {
            bg_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
            hovered_bg_color: Color::TRANSPARENT,
            hovered_text_color: Color::WHITE,
            border_radius: 4.0,
        };

        let mb = menu_bar!((
            Button::new(Text::new("Hardware")).style(about_button_style.get_button_style()),
            {
                menu!((Button::new(Text::new("Use local Pi Hardware"))
                    .style(about_button_style.get_button_style())
                    .width(Length::Fill))(
                    Button::new(Text::new("Connect to remote Pi"))
                        .style(about_button_style.get_button_style())
                        .width(Length::Fill)
                )(
                    Button::new(Text::new("Search for Pi's on local network"))
                        .style(about_button_style.get_button_style())
                        .width(Length::Fill)
                )(hardware_button::view(hardware_view)))
                .max_width(180.0)
                .spacing(2.0)
                .offset(10.0)
            }
        ));

        container(
            Row::new()
                .push(version_button())
                .push(mb)
                .push(unsaved_status::view(unsaved_changes))
                .push(iced::widget::Space::with_width(Length::Fill)) // This takes up remaining space
                .push(self.message_row.view().map(Message::InfoRow))
                .spacing(20.0)
                .padding([0.0, 0.0, 0.0, 0.0]),
        )
        .set_background(Color::from_rgb8(45, 45, 45))
        .into()
    }

    pub fn subscription(&self) -> Subscription<MessageRowMessage> {
        self.message_row.subscription()
    }
}
