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
use hw::{BCMPinNumber, BoardPinNumber, GPIOConfig, HardwareDescriptor, PinFunction};
use hw_listener::{HWListenerEvent, HardwareEvent};
use pin_layout::{bcm_pin_layout_view, board_pin_layout_view};

use crate::hw::{LevelChange, PinDescriptionSet};
use crate::layout::Layout;
use crate::pin_state::{PinState, CHART_UPDATES_PER_SECOND};
use crate::version::version;
use crate::views::hardware::hardware_view;

mod custom_widgets;
mod hw;
mod hw_listener;
mod layout;
mod pin_layout;
mod pin_state;
mod version;
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
    Add,
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
    UpdateCharts,
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    pub pin_function_selected: [PinFunction; 40],
    chosen_layout: Layout,
    hardware_description: Option<HardwareDescriptor>,
    listener_sender: Option<Sender<HardwareEvent>>,
    /// Either desired state of an output, or detected state of input.
    /// Note: Indexed by BoardPinNumber -1 (since BoardPinNumbers start at 1)
    pin_states: [PinState; 40],
    pin_descriptions: Option<PinDescriptionSet>,
    toasts: Vec<Toast>,
    show_toast: bool,
    timeout_secs: u64,
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

    async fn save_via_picker(gpio_config: GPIOConfig) -> io::Result<()> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .set_title("Choose file")
            .set_directory(env::current_dir().unwrap())
            .save_file()
            .await
        {
            let path: std::path::PathBuf = handle.path().to_owned();
            let path_str = path.display().to_string();
            gpio_config.save(&path_str)
        } else {
            Ok(())
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
        if let Some(pin_set) = &self.pin_descriptions {
            for (bcm_pin_number, function) in &self.gpio_config.configured_pins {
                if let Some(board_pin_number) = pin_set.bcm_to_board(*bcm_pin_number) {
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
                pin_descriptions: None,     // Until listener is ready
                pin_states: core::array::from_fn(|_| PinState::new()),
                toasts: Vec::new(),
                show_toast: false,
                timeout_secs: toast::DEFAULT_TIMEOUT,
            },
            Command::perform(Self::load(env::args().nth(1)), |result| match result {
                Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                _ => Message::None,
            }),
        )
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Activate(pin_number) => println!("Pin {pin_number} clicked"),
            Message::PinFunctionSelected(board_pin_number, bcm_pin_number, pin_function) => {
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
            }
            Message::Save => {
                let gpio_config = self.gpio_config.clone();
                return Command::perform(
                    Self::save_via_picker(gpio_config),
                    |result| match result {
                        Ok(_) => Message::None,
                        _ => Message::None, // eprintln ! ("Error saving configuration to {}: {}", path_str, err);
                    },
                );
            }
            Message::Load => {
                return Command::perform(Self::load_via_picker(), |result| match result {
                    Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                    _ => Message::None,
                })
            }
            Message::None => {}
            Message::HardwareListener(event) => match event {
                HWListenerEvent::Ready(config_change_sender, hw_desc, pins) => {
                    self.listener_sender = Some(config_change_sender);
                    self.hardware_description = Some(hw_desc);
                    self.pin_descriptions = Some(pins);
                    self.set_pin_functions_after_load();
                    self.update_hw_config();
                }
                HWListenerEvent::InputChange(bcm_pin_number, level_change) => {
                    if let Some(pins) = &self.pin_descriptions {
                        if let Some(board_pin_number) = pins.bcm_to_board(bcm_pin_number) {
                            self.set_pin_level_change(board_pin_number, level_change);
                        }
                    }
                }
            },

            Message::ChangeOutputLevel(bcm_pin_number, level_change) => {
                if let Some(pins) = &self.pin_descriptions {
                    if let Some(board_pin_number) = pins.bcm_to_board(bcm_pin_number) {
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
                ToastMessage::Add => {
                    self.toasts.clear();
                    self.toasts.push(Toast {
                        title: "About Piggui".into(),
                        body: version(),
                        status: Status::Primary,
                    });
                    self.show_toast = true;
                }
                ToastMessage::Close(index) => {
                    self.show_toast = false;
                    self.toasts.remove(index);
                }
                ToastMessage::Timeout(timeout) => {
                    self.timeout_secs = timeout as u64;
                }
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let layout_selector = pick_list(
            &Layout::ALL[..],
            Some(self.chosen_layout),
            Message::LayoutChanged,
        )
        .width(Length::Shrink)
        .placeholder("Choose Layout");

        let mut main_row = Row::new();

        if let Some(hw_desc) = &self.hardware_description {
            let layout_row = Row::new()
                .push(layout_selector)
                .align_items(Alignment::Start)
                .spacing(10);

            let hardware_desc_row = Row::new()
                .push(hardware_view(hw_desc))
                .align_items(Alignment::Center);

            let file_button_style = ButtonStyle {
                bg_color: Color::new(0.0, 1.0, 1.0, 1.0),
                text_color: Color::BLACK,
                hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0),
                hovered_text_color: Color::WHITE,
                border_radius: 2.0,
            };

            let about_button_style = ButtonStyle {
                bg_color: Color::TRANSPARENT,
                text_color: Color::WHITE,
                hovered_bg_color: Color::TRANSPARENT,
                hovered_text_color: Color::new(0.7, 0.7, 0.7, 1.0),
                border_radius: 4.0,
            };

            let mut configuration_column = Column::new().align_items(Alignment::Start).spacing(10);
            configuration_column = configuration_column.push(layout_row);
            configuration_column = configuration_column.push(hardware_desc_row);
            configuration_column = configuration_column.push(
                Button::new(Text::new("Save Configuration"))
                    .style(file_button_style.get_button_style())
                    .on_press(Message::Save),
            );
            configuration_column = configuration_column.push(
                Button::new(Text::new("Load Configuration"))
                    .style(file_button_style.get_button_style())
                    .on_press(Message::Load),
            );

            let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());
            let add_toast_button = Button::new(version_text)
                .on_press(if !self.show_toast {
                    // Add a new toast if `show_toast` is false
                    Message::Toast(ToastMessage::Add)
                } else {
                    // Close the existing toast if `show_toast` is true
                    let index = self.toasts.len() - 1;
                    Message::Toast(ToastMessage::Close(index))
                })
                .style(about_button_style.get_button_style());

            let version_row = Row::new()
                .push(add_toast_button)
                .align_items(Alignment::Start);

            main_row = main_row.push(
                Column::new()
                    .push(configuration_column)
                    .push(version_row)
                    .align_items(Alignment::Start)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(self.chosen_layout.get_spacing()),
            );
        }

        if let Some(pins) = &self.pin_descriptions {
            let pin_layout = match self.chosen_layout {
                Layout::BoardLayout => board_pin_layout_view(pins, self),
                Layout::BCMLayout => bcm_pin_layout_view(pins, self),
            };

            main_row = main_row.push(
                Column::new()
                    .push(pin_layout)
                    .align_items(Alignment::Center)
                    .height(Length::Fill)
                    .width(Length::Fill),
            );
        }

        let content = container(main_row)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .center_x()
            .center_y()
            .padding(10);

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
        Subscription::batch([
            hw_listener::subscribe().map(Message::HardwareListener),
            iced::time::every(Duration::from_millis(1000 / CHART_UPDATES_PER_SECOND))
                .map(|_| Message::UpdateCharts),
        ])
    }
}
