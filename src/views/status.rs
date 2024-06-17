use crate::custom_widgets::button_style::ButtonStyle;
use crate::{Gpio, Message};
use iced::widget::{Button, Text};
use iced::{Color, Element, Length};

/// There are three types of messages we can display in the message text in the status bar
/// * Error - with an associated long message and a boolean that determines if user must cancel to remove it
/// * Warning - will disappear after a period
/// * Info - will disappear after a period
#[derive(Debug, Clone)]
pub enum StatusMessage {
    Error(String, String, bool),
    Warning(String),
    Info(String),
}

impl StatusMessage {
    fn text(&self) -> String {
        match self {
            StatusMessage::Error(msg, _, _) => msg.clone(),
            StatusMessage::Warning(msg) => msg.clone(),
            StatusMessage::Info(msg) => msg.clone(),
        }
    }
}

#[derive(Default)]
pub struct StatusMessageQueue {
    messages: Vec<StatusMessage>,
}

impl StatusMessageQueue {
    pub fn add(&mut self, message: StatusMessage) {
        self.messages.push(message)
    }

    pub fn peek(&self) -> Option<&StatusMessage> {
        self.messages.last()
    }
}

pub fn status_message(app: &Gpio) -> Element<Message> {
    let button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color: Color::WHITE,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::new(0.7, 0.7, 0.7, 1.0),
        border_radius: 4.0,
    };

    let message = app
        .status_message_queue
        .peek()
        .map(|msg| msg.text())
        .unwrap_or("".into());

    Button::new(Text::new(message))
        .style(button_style.get_button_style())
        .width(Length::Fixed(400.0))
        .into()
}
