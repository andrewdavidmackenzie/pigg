use crate::custom_widgets::button_style::ButtonStyle;
use crate::{Gpio, Message};
use iced::widget::{Button, Text};
use iced::{Color, Element, Length};

/// There are three types of messages we can display in the message text in the status bar.
///
/// They are (in order of priority - highest to lowest):
/// * Error -  will remain until clicked
/// * Warning - will remain until clicked
/// * Info - will disappear after a short time
///
/// Messages of higher priority are shown before those of lower priority.
/// Clicking a message removes it and shows next message.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum StatusMessage {
    Error(String, String) = 2,
    Warning(String) = 1,
    Info(String) = 0,
}

impl StatusMessage {
    fn text(&self) -> String {
        match self {
            StatusMessage::Error(msg, _) => msg.clone(),
            StatusMessage::Warning(msg) => msg.clone(),
            StatusMessage::Info(msg) => msg.clone(),
        }
    }
}

#[derive(Default)]
pub struct StatusMessageQueue {
    queue: Vec<StatusMessage>,
    current_message: Option<StatusMessage>,
}

impl StatusMessageQueue {
    /// Add a new [StatusMessage] to be displayed
    /// If none is being displayed currently, set it as the one that will be displayed by view().
    /// If a message is currently being displayed, add this one to the queue.
    pub fn add_message(&mut self, message: StatusMessage) {
        match self.current_message {
            None => self.current_message = Some(message),
            Some(_) => {
                self.queue.push(message);
                self.queue.sort();
            }
        }
    }

    /// Clear the current message being displayed.
    /// If there is another message in the queue then it sets that as the new message to be shown
    /// If there is no other message queues to be shown, then set to None and no message is shown
    pub fn clear_message(&mut self) {
        if self.queue.is_empty() {
            self.current_message = None;
        } else {
            self.current_message = self.queue.pop();
        }
    }

    /// Are there any [StatusMessage]  of type Info in the queue waiting to be displayed?
    pub fn showing_info_message(&self) -> bool {
        matches!(self.current_message, Some(StatusMessage::Info(_)))
    }
}

pub fn status_message(app: &Gpio) -> Element<Message> {
    let (text_color, message_text) = match &app.status_message_queue.current_message {
        None => (Color::TRANSPARENT, "".into()),
        Some(msg) => {
            let text_color = match msg {
                StatusMessage::Error(_, _) => Color::from_rgb8(255, 0, 0),
                StatusMessage::Warning(_) => iced::Color::new(1.0, 0.647, 0.0, 1.0),
                StatusMessage::Info(_) => Color::WHITE,
            };
            (text_color, msg.text())
        }
    };

    let button_style = ButtonStyle {
        bg_color: Color::TRANSPARENT,
        text_color,
        hovered_bg_color: Color::TRANSPARENT,
        hovered_text_color: Color::WHITE,
        border_radius: 4.0,
    };

    Button::new(Text::new(message_text))
        .on_press(Message::ClearStatusMessage)
        .style(button_style.get_button_style())
        .width(Length::Fixed(400.0))
        .into()
}

#[cfg(test)]
mod test {
    use crate::views::status::StatusMessage::{Error, Info, Warning};
    use crate::views::status::{status_message, StatusMessageQueue};
    use crate::Gpio;
    use iced::application::Application;

    #[test]
    fn errors_first() {
        let mut queue: StatusMessageQueue = Default::default();

        queue.add_message(Info("shown".into()));
        assert!(queue.showing_info_message());
        assert_eq!(queue.current_message, Some(Info("shown".into())));

        // Add three more messages that should be queued up
        queue.add_message(Info("last".into()));
        queue.add_message(Error("first".into(), "Details".into()));
        queue.add_message(Warning("middle".into()));
        assert_eq!(queue.queue.len(), 3);

        // clear the current message, it should be replaced by highest priority message in the queue
        queue.clear_message();
        assert_eq!(
            queue.current_message,
            Some(Error("first".into(), "Details".into()))
        );
        assert_eq!(queue.queue.len(), 2);

        queue.clear_message();
        assert_eq!(queue.current_message, Some(Warning("middle".into())));
        assert_eq!(queue.queue.len(), 1);

        queue.clear_message();
        assert_eq!(queue.current_message, Some(Info("last".into())));
        assert_eq!(queue.queue.len(), 0);
    }

    #[tokio::test]
    async fn no_text() {
        let (app, _command) = Gpio::new(());
        let el = status_message(&app);
        let button = el.as_widget();
        let tree = button.children();
        println!("{:?}", tree);
        let text = tree.first();
        println!("{:?}", text);
    }
}
