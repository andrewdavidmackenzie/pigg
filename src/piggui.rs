use std::time::Duration;
use std::{env, io};

use iced::futures::channel::mpsc::Sender;
use iced::widget::{container, pick_list, Button, Column, Row, Text};
use iced::{
    executor, window, Alignment, Application, Color, Command, Element, Length, Settings,
    Subscription, Theme,
};

use custom_widgets::button_style::ButtonStyle;
use custom_widgets::toast::{self, Manager, Status, Toast};
use hw::{
    hw_listener::{HWListenerEvent, HardwareEvent},
    BCMPinNumber, BoardPinNumber, GPIOConfig, HardwareDescription, PinFunction,
};
use pin_layout::{bcm_pin_layout_view, board_pin_layout_view};

use crate::hw::{hw_listener, LevelChange};
use crate::layout::Layout;
use crate::pin_state::{PinState, CHART_UPDATES_PER_SECOND};
use crate::views::hardware::hw_description;
use crate::views::info::info_row;
use crate::views::status::{StatusMessage, StatusMessageQueue};
use crate::views::version::version;

mod custom_widgets;
mod hw;
mod layout;
mod pin_layout;
mod pin_state;
mod views;

fn main() -> Result<(), iced::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("{}", version());
        return Ok(());
    }

    let layout = Layout::BoardLayout;
    let size = layout.get_window_size();
    let window = window::Settings {
        resizable: true,
        exit_on_close_request: false,
        size,
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum ToastMessage {
    VersionToast,
    HardwareDetailsToast,
    Close(usize),
    Timeout(f64),
}
#[derive(Debug, Clone)]
pub enum Message {
    Activate(BoardPinNumber),
    PinFunctionSelected(BoardPinNumber, BCMPinNumber, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
    HardwareListener(HWListenerEvent),
    ChangeOutputLevel(BCMPinNumber, LevelChange),
    Toast(ToastMessage),
    Save,
    Load,
    ShowStatusMessage(StatusMessage),
    ClearStatusMessage,
    UpdateCharts,
    WindowEvent(iced::Event),
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    pub pin_function_selected: [PinFunction; 40],
    chosen_layout: Layout,
    hardware_description: Option<HardwareDescription>,
    listener_sender: Option<Sender<HardwareEvent>>,
    /// Either desired state of an output, or detected state of input.
    /// Note: Indexed by BoardPinNumber -1 (since BoardPinNumbers start at 1)
    pin_states: [PinState; 40],
    pub toasts: Vec<Toast>,
    pub show_toast: bool,
    timeout_secs: u64,
    unsaved_changes: bool,
    pending_load: bool,
    pending_exit: bool,
    status_message_queue: StatusMessageQueue,
}

impl Gpio {
    async fn load(filename: Option<String>) -> io::Result<Option<(String, GPIOConfig)>> {
        match filename {
            Some(config_filename) => {
                let config = GPIOConfig::load(&config_filename)?;
                Ok(Some((config_filename, config)))
            }
            None => Ok(None),
        }
    }

    async fn load_via_picker() -> io::Result<Option<(String, GPIOConfig)>> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Pigg Config", &["pigg"])
            .set_title("Choose config file to load")
            .set_directory(env::current_dir().unwrap())
            .pick_file()
            .await
        {
            let path: std::path::PathBuf = handle.path().to_owned();
            let path_str = path.display().to_string();
            Self::load(Some(path_str)).await
        } else {
            Ok(None)
        }
    }

    async fn save_via_picker(gpio_config: GPIOConfig) -> io::Result<String> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .add_filter("Pigg Config", &["pigg"])
            .set_title("Choose file")
            .set_directory(env::current_dir().unwrap())
            .save_file()
            .await
        {
            let path: std::path::PathBuf = handle.path().to_owned();
            let path_str = path.display().to_string();
            gpio_config.save(&path_str)
        } else {
            Ok("File save cancelled".into())
        }
    }

    /// Send the GPIOConfig from the GUI to the hardware to have it applied
    fn update_hw_config(&mut self) {
        if let Some(ref mut listener) = &mut self.listener_sender {
            let _ = listener.try_send(HardwareEvent::NewConfig(self.gpio_config.clone()));
        }
    }

    /// A new function has been selected for a pin via the UI, this function:
    /// - updates the pin_selected_function array for the UI
    /// - saves it in the gpio_config, so when we save later it's there
    /// - sends the update to the hardware to have it applied
    fn new_pin_function(
        &mut self,
        board_pin_number: BoardPinNumber,
        bcm_pin_number: BCMPinNumber,
        new_function: PinFunction,
    ) {
        let board_pin_index = board_pin_number as usize - 1;
        let previous_function = self.pin_function_selected[board_pin_index];
        if new_function != previous_function {
            self.pin_function_selected[board_pin_index] = new_function;
            // Pushing selected pin to the Pin Config
            if let Some(pin_config) = self
                .gpio_config
                .configured_pins
                .iter_mut()
                .find(|(pin, _)| *pin == bcm_pin_number)
            {
                *pin_config = (bcm_pin_number, new_function);
            } else {
                // TODO this could just be adding to the config, not replacing existing ones, no?
                // Add a new configuration entry if it doesn't exist
                self.gpio_config
                    .configured_pins
                    .push((bcm_pin_number, new_function));
            }
            // Report config changes to the hardware listener
            // Since config loading and hardware listener setup can occur out of order
            // mark the config as changed. If we send to the listener, then mark as done
            if let Some(ref mut listener) = &mut self.listener_sender {
                let _ =
                    listener.try_send(HardwareEvent::NewPinConfig(bcm_pin_number, new_function));
            }
        }
    }

    /// Go through all the pins in the loaded GPIOConfig and set its function in the
    /// pin_function_selected array, which is what is used for drawing the UI correctly.
    fn set_pin_functions_after_load(&mut self) {
        if let Some(hardware_description) = &self.hardware_description {
            for (bcm_pin_number, function) in &self.gpio_config.configured_pins {
                if let Some(board_pin_number) =
                    hardware_description.pins.bcm_to_board(*bcm_pin_number)
                {
                    self.pin_function_selected[board_pin_number as usize - 1] = *function;

                    // For output pins, if there is an initial state set then set that in pin state
                    // so the toggler will be drawn correctly on first draw
                    if let PinFunction::Output(Some(level)) = function {
                        self.pin_states[board_pin_number as usize - 1]
                            .set_level(LevelChange::new(*level));
                    }
                }
            }
        }
    }

    /// Set the pin (using board number) level with a LevelChange
    fn set_pin_level_change(
        &mut self,
        board_pin_number: BoardPinNumber,
        level_change: LevelChange,
    ) {
        self.pin_states[board_pin_number as usize - 1].set_level(level_change);
    }

    fn configuration_column(&self) -> Element<'static, Message> {
        let layout_selector = pick_list(
            &Layout::ALL[..],
            Some(self.chosen_layout),
            Message::LayoutChanged,
        )
        .width(Length::Shrink)
        .placeholder("Choose Layout");

        let layout_row = Row::new()
            .push(layout_selector)
            .align_items(Alignment::Start)
            .spacing(10);

        let file_button_style = ButtonStyle {
            bg_color: Color::new(0.0, 1.0, 1.0, 1.0),
            text_color: Color::BLACK,
            hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0),
            hovered_text_color: Color::WHITE,
            border_radius: 2.0,
        };

        let mut configuration_column = Column::new().align_items(Alignment::Start).spacing(10);
        configuration_column = configuration_column.push(layout_row);
        configuration_column = configuration_column.push(
            Button::new(Text::new("Save Configuration"))
                .style(file_button_style.get_button_style())
                .on_press(Message::Save),
        );
        configuration_column = configuration_column.push(
            Button::new(Text::new("Load Configuration"))
                .style(file_button_style.get_button_style())
                .on_press(if !self.show_toast {
                    // Add a new toast if `show_toast` is false
                    Message::Load
                } else {
                    // Close the existing toast if `show_toast` is true
                    let index = self.toasts.len() - 1;
                    Message::Toast(ToastMessage::Close(index))
                }),
        );

        configuration_column.into()
    }

    fn main_row(&self) -> Element<Message> {
        let mut main_row = Row::new();

        main_row = main_row.push(
            Column::new()
                .push(self.configuration_column())
                .align_items(Alignment::Start)
                .width(Length::Shrink)
                .height(Length::Shrink)
                .spacing(self.chosen_layout.get_spacing()),
        );

        if let Some(hw_description) = &self.hardware_description {
            let pin_layout = match self.chosen_layout {
                Layout::BoardLayout => board_pin_layout_view(&hw_description.pins, self),
                Layout::BCMLayout => bcm_pin_layout_view(&hw_description.pins, self),
            };

            main_row = main_row.push(
                Column::new()
                    .push(pin_layout)
                    .align_items(Alignment::Center)
                    .height(Length::Fill)
                    .width(Length::Fill),
            );
        }

        main_row.into()
    }
}

impl Application for Gpio {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Gpio, Command<Self::Message>) {
        (
            Self {
                config_filename: None,
                gpio_config: GPIOConfig::default(),
                pin_function_selected: [PinFunction::None; 40],
                chosen_layout: Layout::BoardLayout,
                hardware_description: None, // Until listener is ready
                listener_sender: None,      // Until listener is ready
                pin_states: core::array::from_fn(|_| PinState::new()),
                toasts: Vec::new(),
                show_toast: false,
                timeout_secs: toast::DEFAULT_TIMEOUT,
                unsaved_changes: false,
                pending_load: false,
                pending_exit: false,
                status_message_queue: Default::default(),
            },
            Command::perform(Self::load(env::args().nth(1)), |result| match result {
                Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                _ => Message::None,
            }),
        )
    }

    fn title(&self) -> String {
        self.config_filename
            .clone()
            .unwrap_or(String::from("Piggui"))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::WindowEvent(event) => {
                if let iced::Event::Window(window::Id::MAIN, window::Event::CloseRequested) = event
                {
                    if self.unsaved_changes {
                        self.toasts.clear();
                        self.toasts.push(Toast {
                            title: "Unsaved Changes".into(),
                            body: "You have unsaved changes. Do you want to exit without saving?"
                                .into(),
                            status: Status::Danger,
                        });
                        self.show_toast = true;
                        self.pending_exit = true;
                    } else {
                        return window::close(window::Id::MAIN);
                    }
                }
            }
            Message::Activate(pin_number) => println!("Pin {pin_number} clicked"),

            Message::PinFunctionSelected(board_pin_number, bcm_pin_number, pin_function) => {
                self.unsaved_changes = true;
                self.new_pin_function(board_pin_number, bcm_pin_number, pin_function);
            }

            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
                let layout_size = layout.get_window_size();
                return window::resize(window::Id::MAIN, layout_size);
            }

            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename);
                self.gpio_config = config;
                self.set_pin_functions_after_load();
                self.update_hw_config();
                self.unsaved_changes = true;
            }

            Message::Save => {
                let gpio_config = self.gpio_config.clone();
                self.unsaved_changes = false;
                return Command::perform(
                    Self::save_via_picker(gpio_config),
                    |result| match result {
                        Ok(_) => Message::ShowStatusMessage(StatusMessage::Info(
                            "File saved successfully".to_string(),
                        )),
                        Err(e) => Message::ShowStatusMessage(StatusMessage::Error(
                            "Error saving file".to_string(),
                            format!("Error saving file. {e}",),
                        )),
                    },
                );
            }

            Message::Load => {
                if self.unsaved_changes {
                    self.toasts.clear();
                    self.toasts.push(Toast {
                        title: "Unsaved Changes".into(),
                        body: "You have unsaved changes. Do you want to continue without saving?"
                            .into(),
                        status: Status::Danger,
                    });
                    self.show_toast = true;
                    self.pending_load = true;
                } else {
                    return Command::perform(Self::load_via_picker(), |result| match result {
                        Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                        _ => Message::None,
                    });
                }
            }

            Message::None => {}

            Message::HardwareListener(event) => match event {
                HWListenerEvent::Ready(config_change_sender, hw_desc) => {
                    self.listener_sender = Some(config_change_sender);
                    self.hardware_description = Some(hw_desc);
                    self.set_pin_functions_after_load();
                    self.update_hw_config();
                }
                HWListenerEvent::InputChange(bcm_pin_number, level_change) => {
                    if let Some(hardware_description) = &self.hardware_description {
                        if let Some(board_pin_number) =
                            hardware_description.pins.bcm_to_board(bcm_pin_number)
                        {
                            self.set_pin_level_change(board_pin_number, level_change);
                        }
                    }
                }
            },

            Message::ChangeOutputLevel(bcm_pin_number, level_change) => {
                if let Some(hardware_description) = &self.hardware_description {
                    if let Some(board_pin_number) =
                        hardware_description.pins.bcm_to_board(bcm_pin_number)
                    {
                        self.set_pin_level_change(board_pin_number, level_change.clone());
                    }
                    if let Some(ref mut listener) = &mut self.listener_sender {
                        let _ = listener.try_send(HardwareEvent::OutputLevelChanged(
                            bcm_pin_number,
                            level_change,
                        ));
                    }
                }
            }

            Message::UpdateCharts => {
                // Update all the charts of the pins that have an assigned function
                for pin in 0..40 {
                    if self.pin_function_selected[pin] != PinFunction::None {
                        self.pin_states[pin].chart.refresh();
                    }
                }
            }

            Message::Toast(toast_message) => match toast_message {
                ToastMessage::VersionToast => {
                    self.toasts.clear();
                    self.toasts.push(Toast {
                        title: "About Piggui".into(),
                        body: version(),
                        status: Status::Primary,
                    });
                    self.show_toast = true;
                }
                ToastMessage::HardwareDetailsToast => {
                    self.toasts.clear();
                    self.toasts.push(Toast {
                        title: "About Connected Hardware".into(),
                        body: hw_description(self),
                        status: Status::Primary,
                    });
                    self.show_toast = true;
                }
                ToastMessage::Close(index) => {
                    self.show_toast = false;
                    self.toasts.remove(index);
                    if self.pending_load {
                        self.pending_load = false;
                        return Command::perform(Self::load_via_picker(), |result| match result {
                            Ok(Some((filename, config))) => {
                                Message::ConfigLoaded((filename, config))
                            }
                            _ => Message::None,
                        });
                    }
                    if self.pending_exit {
                        self.pending_exit = false;
                        return window::close(window::Id::MAIN);
                    }
                }
                ToastMessage::Timeout(timeout) => {
                    self.timeout_secs = timeout as u64;
                }
            },

            Message::ShowStatusMessage(msg) => self.status_message_queue.add_message(msg),

            Message::ClearStatusMessage => self.status_message_queue.clear_message(),
        }
        Command::none()
    }

    /*
       +-window-------------------------------------------------------------------------------+
       |  +-content(main_col)----------------------------------------------------------------+ |
       |  | +-main-row--------------------------------------------------------------------+ | |
       |  | | +-configuration-column-+--------------------------------------------------+ | | |
       |  | | |                      |                                                  | | | |
       |  | | +-configuration-column-+--------------------------------------------------+ | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  | +-info-row--------------------------------------------------------------------+ | |
       |  | |  <version> | <hardware> | <status>                                          | | |
       |  | +-----------------------------------------------------------------------------+ | |
       |  +---------------------------------------------------------------------------------+ |
       +--------------------------------------------------------------------------------------+
    */
    fn view(&self) -> Element<Self::Message> {
        let main_col = Column::new().push(self.main_row()).push(info_row(self));

        let content = container(main_col)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y()
            .padding([10.0, 10.0, 0.0, 10.0]);

        Manager::new(content, &self.toasts, |index| {
            Message::Toast(ToastMessage::Close(index))
        })
        .timeout(self.timeout_secs)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            hw_listener::subscribe().map(Message::HardwareListener),
            iced::time::every(Duration::from_millis(1000 / CHART_UPDATES_PER_SECOND))
                .map(|_| Message::UpdateCharts),
            iced::event::listen().map(Message::WindowEvent),
        ];

        if self.status_message_queue.showing_info_message() {
            println!("Will clear info message in 3s");
            subscriptions.push(
                iced::time::every(Duration::from_secs(3)).map(|_| Message::ClearStatusMessage),
            );
        }

        Subscription::batch(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_add_toast_message() {
        let mut app = Gpio::new(()).0;

        // No toasts should be present
        assert!(app.toasts.is_empty());

        // Add a toast
        let _ = app.update(Message::Toast(ToastMessage::VersionToast));

        // Check if a toast was added
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
        assert_eq!(toast.title, "About Piggui");
    }

    #[tokio::test]
    async fn test_close_toast_message() {
        let mut app = Gpio::new(()).0;

        // Add a toast
        let _ = app.update(Message::Toast(ToastMessage::VersionToast));

        // Ensure the toast was added
        assert_eq!(app.toasts.len(), 1);

        // Close the toast
        let _ = app.update(Message::Toast(ToastMessage::Close(0)));

        // Check if the toast was removed
        assert!(app.toasts.is_empty());
    }

    #[tokio::test]
    async fn test_window_close_with_unsaved_changes() {
        let mut app = Gpio::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a close window event
        let _ = app.update(Message::WindowEvent(iced::Event::Window(
            window::Id::MAIN,
            window::Event::CloseRequested,
        )));

        // Check if a warning toast was added
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to exit without saving?"
        );
    }

    #[tokio::test]
    async fn test_load_with_unsaved_changes() {
        let mut app = Gpio::new(()).0;

        // Simulate unsaved changes
        app.unsaved_changes = true;

        // Send a load message
        let _ = app.update(Message::Load);

        // Check if a warning toast was added
        assert_eq!(app.toasts.len(), 1);
        let toast = &app.toasts[0];
        assert_eq!(toast.title, "Unsaved Changes");
        assert_eq!(
            toast.body,
            "You have unsaved changes. Do you want to continue without saving?"
        );
    }

    #[tokio::test]
    async fn test_toast_timeout() {
        let mut app = Gpio::new(()).0;

        // Send a timeout message
        let _ = app.update(Message::Toast(ToastMessage::Timeout(5.0)));

        // Check the timeout
        assert_eq!(app.timeout_secs, 5);
    }
}
