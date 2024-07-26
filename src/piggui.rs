use std::str::FromStr;
use crate::file_helper::{maybe_load_no_picker, pick_and_load, save};
use crate::hw::config::HardwareConfig;
use crate::toast_handler::{ToastHandler, ToastMessage};
use crate::views::hardware_view::HardwareViewMessage::NewConfig;
use crate::views::hardware_view::{HardwareView, HardwareViewMessage};
use crate::views::info_row::InfoRow;
use crate::views::layout_selector::{Layout, LayoutSelector};
use crate::views::main_row;
use crate::views::message_row::MessageRowMessage::ShowStatusMessage;
use crate::views::message_row::{MessageMessage, MessageRowMessage};
use crate::Message::*;
use clap::{Arg, ArgMatches};
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::{self, column, container, row, text, text_input, Button, Column, Text};
use iced::{
    executor, window, Application, Color, Command, Element, Length, Settings, Subscription, Theme,
};

use crate::styles::button_style::ButtonStyle;
use crate::styles::container_style::ContainerStyle;
use crate::widgets::modal::Modal;
use views::pin_state::PinState;

use iced::event::{self, Event};
use iroh_net::NodeId;
use iroh_net::relay::RelayUrl;
use crate::styles::text_style::TextStyle;

mod file_helper;
#[cfg(any(feature = "fake_hw", feature = "pi_hw"))]
pub mod hardware_subscription;
mod hw;
pub mod network_subscription;
mod styles;
mod toast_handler;
mod views;
mod widgets;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: true,
        exit_on_close_request: false,
        size: LayoutSelector::get_default_window_size(),
        ..Default::default()
    };

    Piggui::run(Settings {
        window,
        ..Default::default()
    })
}

/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about("'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware");

    let app = app.arg(
        Arg::new("nodeid")
            .short('n')
            .long("nodeid")
            .num_args(1)
            .number_of_values(1)
            .value_name("NODEID")
            .help("Node Id of a piglet instance to connect to"),
    );

    let app = app.arg(
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}

/// These are the messages that Piggui responds to
#[derive(Debug, Clone)]
pub enum Message {
    ConfigLoaded(String, HardwareConfig),
    ConfigSaved,
    ConfigChangesMade,
    Save,
    Load,
    LayoutChanged(Layout),
    Hardware(HardwareViewMessage),
    Toast(ToastMessage),
    InfoRow(MessageRowMessage),
    WindowEvent(iced::Event),
    HardwareLost,
    MenuBarButtonClicked,
    ShowModal,
    HideModal,
    Submit,
    ConnectionId(String),
    RelayURL(String),
    ModalKeyEvent(Event),
    ConnectIroh(String, String),
}

/// [Piggui] Is the struct that holds application state and implements [Application] for Iced
pub struct Piggui {
    config_filename: Option<String>,
    layout_selector: LayoutSelector,
    unsaved_changes: bool,
    info_row: InfoRow,
    toast_handler: ToastHandler,
    hardware_view: HardwareView,
    show_modal: bool,
    connection_id: String,
    relay_url: String,
    connection_error: String,
}

async fn empty() {}
impl Piggui {
    fn hide_modal(&mut self) {
        self.show_modal = false;
        self.connection_id.clear();
    }
}
impl Application for Piggui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Piggui, Command<Message>) {
        let matches = get_matches();
        let config_filename = matches
            .get_one::<String>("config-file")
            .map(|s| s.to_string());

        let nodeid = matches.get_one::<String>("nodeid").map(|s| s.to_string());

        // TODO this will come from UI entry later. For now copy this from the output of piglet then run piggui
        //let node_id = "2r7vxyfvkfgwfkcxt5wky72jghy4n6boawnvz5fxes62tqmnnmhq";

        (
            Self {
                config_filename: None,
                layout_selector: LayoutSelector::new(),
                unsaved_changes: false,
                info_row: InfoRow::new(),
                toast_handler: ToastHandler::new(),
                hardware_view: HardwareView::new(nodeid),
                show_modal: false,
                connection_id: String::new(),
                relay_url: String::new(),
                connection_error: String::new(),
            },
            maybe_load_no_picker(config_filename),
        )
    }

    fn title(&self) -> String {
        self.config_filename
            .clone()
            .unwrap_or(String::from("Piggui"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            WindowEvent(event) => {
                if let iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) = event
                {
                    if self.unsaved_changes {
                        let _ = self
                            .toast_handler
                            .update(ToastMessage::UnsavedChangesExitToast, &self.hardware_view);
                        self.unsaved_changes = false;
                    } else {
                        return window::close(window::Id::MAIN);
                    }
                }
            }

            ConnectIroh(connection_id, relay_url) => {

                if connection_id.trim().is_empty() {
                    self.connection_error = String::from("Pls Enter Node Id");
                    return Command::none();
                }
                if relay_url.trim().is_empty() {
                    self.connection_error = String::from("Pls Enter Relay Url");
                    return Command::none();
                }

                let node_id_result = NodeId::from_str(connection_id.as_str());
                match node_id_result {
                    Ok(_node_id) => {
                        let relay_url_result = RelayUrl::from_str(relay_url.as_str());
                        match relay_url_result {
                            Ok(_relay_url) => {
                                // TODO
                                // Make iroh connection
                                // Add spinner when establishing remote connection
                            }
                            Err(err) => {
                                self.connection_error = format!("{}", err);
                            }
                        }
                    }
                    Err(err) => {
                        self.connection_error = format!("{}", err);
                    }
                }

                return Command::none();
            }

            ModalKeyEvent(event) => {
                return match event {
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Tab),
                        modifiers,
                        ..
                    }) => {
                        if modifiers.shift() {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(key::Named::Escape),
                        ..
                    }) => {
                        self.hide_modal();
                        Command::none()
                    }
                    _ => Command::none(),
                }
            }

            ConnectionId(connection_id) => {
                self.connection_id = connection_id;
                return Command::none();
            }

            RelayURL(relay_url) => {
                self.relay_url = relay_url;
                return Command::none();
            }

            Submit => {
                if !self.connection_id.is_empty() {
                    self.hide_modal();
                }
                return Command::none();
            }

            ShowModal => {
                self.show_modal = true;
                return iced::widget::focus_next();
            }
            HideModal => {
                self.hide_modal();
                self.connection_error.clear();
                return Command::none();
            }

            MenuBarButtonClicked => {
                return Command::none();
            }

            LayoutChanged(layout) => {
                // Keep overall window management at this level and out of LayoutSelector
                return window::resize(window::Id::MAIN, self.layout_selector.update(layout));
            }

            Save => return save(self.hardware_view.get_config()),

            ConfigSaved => {
                self.unsaved_changes = false;
                return Command::perform(empty(), |_| {
                    InfoRow(ShowStatusMessage(MessageMessage::Info(
                        "File saved successfully".to_string(),
                    )))
                });
            }

            Load => {
                if self.unsaved_changes {
                    let _ = self
                        .toast_handler
                        .update(ToastMessage::UnsavedChangesToast, &self.hardware_view);
                } else {
                    return Command::batch(vec![ToastHandler::clear_last_toast(), pick_and_load()]);
                }
            }

            Toast(toast_message) => {
                return self
                    .toast_handler
                    .update(toast_message, &self.hardware_view);
            }

            InfoRow(msg) => return self.info_row.update(msg),

            Hardware(msg) => return self.hardware_view.update(msg),

            ConfigChangesMade => self.unsaved_changes = true,

            ConfigLoaded(filename, config) => {
                self.config_filename = Some(filename);
                self.unsaved_changes = false;
                return Command::perform(empty(), |_| Hardware(NewConfig(config)));
            }
            HardwareLost => {
                return Command::perform(empty(), |_| {
                    InfoRow(ShowStatusMessage(MessageMessage::Error(
                        "Connection to Hardware Lost".to_string(),
                        "The connection to GPIO hardware has been lost. Check networking and try to re-connect".to_string()
                    )))
                });
            }
        }

        Command::none()
    }

    /*
       +-window-------------------------------------------------------------------------------+
       |  +-content(main_col)---------------------------------------------------------------+ |
       |  | +-main-row--------------------------------------------------------------------+ | |
       |  | |                                                                             | | |
       |  | |                                                                             | | |
       |  | |                                                                             | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  | +-info-row--------------------------------------------------------------------+ | |
       |  | |                                                                             | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  +---------------------------------------------------------------------------------+ |
       +--------------------------------------------------------------------------------------+
    */
    fn view(&self) -> Element<Message> {
        let main_col = Column::new()
            .push(main_row::view(&self.hardware_view, &self.layout_selector))
            .push(
                self.info_row
                    .view(self.unsaved_changes, &self.hardware_view),
            );

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y();

        if self.show_modal {
            let modal_connect_button_style = ButtonStyle {
                bg_color: Color::new(0.0, 1.0, 1.0, 1.0), // Cyan background color
                text_color: Color::BLACK,
                hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0), // Darker cyan color when hovered
                hovered_text_color: Color::WHITE,
                border_radius: 2.0,
            };

            let modal_cancel_button_style = ButtonStyle {
                bg_color: Color::new(0.8, 0.0, 0.0, 1.0), // Gnome like Red background color
                text_color: Color::WHITE,
                hovered_bg_color: Color::new(0.9, 0.2, 0.2, 1.0), // Slightly lighter red when hovered
                hovered_text_color: Color::WHITE,
                border_radius: 2.0,
            };

            let modal_container_style = ContainerStyle {
                border_color: Color::WHITE,
            };

            let connection_error_style = TextStyle {
                text_color: Color::new(1.0, 0.0, 0.0, 0.5)
            };

            let modal = container(
                column![
                    text("Connect To Remote Pi").size(20),
                    column![
                        column![
                            text(self.connection_error.clone()).style(connection_error_style.get_text_color()),
                            text("Node Id").size(12),
                            text_input("Enter node id", &self.connection_id)
                                .on_input(Message::ConnectionId)
                                .on_submit(Message::Submit)
                                .padding(5),
                        ]
                        .spacing(5),
                        column![
                            text("Relay URL").size(12),
                            text_input("Enter Relay Url", &self.relay_url)
                                .on_input(Message::RelayURL)
                                .on_submit(Message::Submit)
                                .padding(5),
                        ]
                        .spacing(5),
                        row![
                            Button::new(Text::new("Cancel"))
                                .on_press(Message::HideModal)
                                .style(modal_cancel_button_style.get_button_style()),
                            Button::new(Text::new("Connect"))
                                .on_press(Message::ConnectIroh(self.connection_id.clone(), self.relay_url.clone()))
                                .style(modal_connect_button_style.get_button_style())
                        ]
                        .spacing(360),
                    ]
                    .spacing(10)
                ]
                .spacing(20),
            )
            .style(modal_container_style.get_container_style())
            .width(520)
            .padding(15);

            Modal::new(content, modal)
                .on_blur(Message::HideModal)
                .into()
        } else {
            self.toast_handler.view(content.into())
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Subscribe to events from Hardware, from Windows and timings for StatusRow
    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![
            iced::event::listen().map(WindowEvent),
            event::listen().map(ModalKeyEvent),
            self.info_row.subscription().map(InfoRow),
            self.hardware_view.subscription().map(Hardware),
        ];

        Subscription::batch(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_close_with_unsaved_changes() {
        let mut app = Piggui::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a close window event
        let _ = app.update(WindowEvent(iced::Event::Window(
            window::Id::MAIN,
            window::Event::CloseRequested,
        )));

        // Check if a warning toast was added
        assert_eq!(app.toast_handler.get_toasts().len(), 1);
        let toast = &app.toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to exit without saving?"
        );
    }

    #[test]
    fn test_load_with_unsaved_changes() {
        let mut app = Piggui::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a load message
        let _ = app.update(Load);

        // Check if a warning toast was added
        assert_eq!(app.toast_handler.get_toasts().len(), 1);
        let toast = &app.toast_handler.get_toasts()[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to continue without saving?"
        );
    }
}
