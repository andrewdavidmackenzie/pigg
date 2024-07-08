use crate::styles::background::SetAppearance;
use crate::styles::button_style::ButtonStyle;
use crate::views::hardware_view::HardwareView;
use crate::views::message_row::{MessageRow, MessageRowMessage};
use crate::views::version::version_button;
use crate::views::{hardware_button, unsaved_status};
use crate::Message;
use iced::widget::{container, Button, Row, Text};
use iced::{Color, Command, Element, Length};
use iced_aw::menu::{Item,  StyleSheet};
use iced_aw::style::MenuBarStyle;
use iced_aw::{menu, menu_bar};
use iced_futures::core::Background;
use iced_futures::Subscription;

const MENU_WIDTH: f32 = 200.0;

const ENABLED_MENU_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::WHITE,
    hovered_bg_color: Color::TRANSPARENT,
    hovered_text_color: Color::WHITE,
    border_radius: 4.0,
};

const DISABLED_MENU_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    bg_color: Color::TRANSPARENT,
    text_color: Color::from_rgb(0.5, 0.5, 0.5), // Medium grey text color
    hovered_bg_color: Color::from_rgb(0.2, 0.2, 0.2),
    hovered_text_color: Color::from_rgb(0.5, 0.5, 0.5),
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
        let mb = menu_bar!((
            Button::new(Text::new("Show Hardware Details"))
                .style(ENABLED_MENU_BUTTON_STYLE.get_button_style()),
            {
                // Conditionally render menu items based on hardware features
                menu!((menu_button(
                    "Use local Pi Hardware".to_string(),
                    cfg!(feature = "pi_hw"),
                ))(menu_button(
                    "Connect to remote Pi...".to_string(),
                    cfg!(feature = "network"),
                ))(menu_button(
                    "Search for Pi's on local network".to_string(),
                    cfg!(feature = "network"),
                ))(hardware_button::view(hardware_view)))
                .width(MENU_WIDTH)
                .spacing(2.0)
                .offset(10.0)
            }
        ))
        .style(|theme: &iced::Theme| menu::Appearance {
            bar_background: Background::Color(Color::TRANSPARENT),
            ..theme.appearance(&MenuBarStyle::Default)
        });

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

fn menu_button(text: String, enabled: bool) -> Button<'static, Message> {
    let button_style = if enabled {
        ENABLED_MENU_BUTTON_STYLE.get_button_style()
    } else {
        DISABLED_MENU_BUTTON_STYLE.get_button_style()
    };

    Button::new(Text::new(text))
        .style(button_style)
        .width(Length::Fill)
}
