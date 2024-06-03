use std::{env, io};

use iced::{
    alignment, Alignment, Application, Color, Command, Element, executor, Length, Settings, Subscription,
    Theme, window,
};
use iced::futures::channel::mpsc::Sender;
use iced::widget::{Button, Column, container, pick_list, Row, Text};

use hw::{BCMPinNumber, BoardPinNumber, GPIOConfig, PinDescription, PinFunction, PinLevel};
use hw::HardwareDescriptor;
use hw::InputPull;
// Importing pin layout views
use hw_listener::{HardwareEvent, HWListenerEvent};
use pin_layout::{bcm_pin_layout_view, board_pin_layout_view, select_pin_function};
use style::CustomButton;

mod hw;
mod pin_layout;
mod style;
mod custom_widgets {
    pub mod circle;
    pub mod led;
    pub mod line;
}
mod hw_listener;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    BoardLayout,
    BCMLayout,
}

impl Layout {
    const ALL: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];
}

// Implementing format for Layout
// TODO could maybe put the Name as a &str inside the enum elements above?
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layout::BoardLayout => "Board Pin Layout",
                Layout::BCMLayout => "BCM Pin Layout",
            }
        )
    }
}

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        decorations: true,
        size: iced::Size::new(1000.0, 900.0),
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    Activate(BoardPinNumber),
    PinFunctionSelected(BoardPinNumber, BCMPinNumber, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
    HardwareListener(HWListenerEvent),
    ChangeOutputLevel(BCMPinNumber, PinLevel),
    Save,
    Load,
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    pub pin_function_selected: [PinFunction; 40],
    chosen_layout: Layout,
    hardware_description: Option<HardwareDescriptor>,
    listener_sender: Option<Sender<HardwareEvent>>,
    /// Either desired state or output, or detected state of input. Note BCMPinNumber, that starts
    /// at 0 (GPIO0)
    pin_states: [Option<PinLevel>; 40],
    pin_descriptions: Option<[PinDescription; 40]>,
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
            .set_directory(std::env::current_dir().unwrap())
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
            .set_directory(std::env::current_dir().unwrap())
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

    // Send the Config from the GUI to the hardware to have it applied
    fn update_hw_config(&mut self) {
        if let Some(ref mut listener) = &mut self.listener_sender {
            let _ = listener.try_send(HardwareEvent::NewConfig(self.gpio_config.clone()));
        }
    }

    // A new function has been selected for a pin via the UI
    fn new_pin_function(
        &mut self,
        board_pin_number: BoardPinNumber,
        bcm_pin_number: BCMPinNumber,
        new_function: PinFunction,
    ) {
        let pin_index = board_pin_number as usize - 1;
        let previous_function = self.pin_function_selected[pin_index];
        if new_function != previous_function {
            self.pin_function_selected[pin_index] = new_function;
            // Pushing selected pin to the Pin Config
            if let Some(pin_config) = self
                .gpio_config
                .configured_pins
                .iter_mut()
                .find(|(pin, _)| *pin == bcm_pin_number)
            {
                *pin_config = (bcm_pin_number, new_function);
            } else {
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
    // TODO this has a lot in common with bcm_pin_layout_view() in pin_layout.rs see if we can merge
    // TODO or factor out a function - maybe improve data structures as we have a bit of
    // TODO repitition - use a map to find by BCM pin number or something
    fn set_pin_functions_after_load(&mut self) {
        if let Some(pins) = &self.pin_descriptions {
            let gpio_pins = pins
                .iter()
                .filter(|pin| pin.options.len() > 1)
                .filter(|pin| pin.bcm_pin_number.is_some())
                .collect::<Vec<&PinDescription>>();

            for pin in gpio_pins {
                if let Some(function) = select_pin_function(pin, &self.gpio_config, self) {
                    self.pin_function_selected[pin.board_pin_number as usize - 1] = function;

                    // For output pins, if there is an initial state set then set that in pin state
                    // so the toggler will be drawn correctly on first draw
                    if let PinFunction::Output(level) = function {
                        match pin.bcm_pin_number {
                            None => {}
                            Some(bcm) => self.pin_states[bcm as usize] = level,
                        };
                    }
                }
            }
        }
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
                pin_states: [None; 40],
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
            }
            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename);
                self.gpio_config = config.clone();
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
                HWListenerEvent::InputChange(level_change) => {
                    self.pin_states[level_change.bcm_pin_number as usize] =
                        Some(level_change.new_level);
                }
            },
            Message::ChangeOutputLevel(bcm_pin_number, new_level) => {
                self.pin_states[bcm_pin_number as usize] = Some(new_level);
                if let Some(ref mut listener) = &mut self.listener_sender {
                    let _ = listener
                        .try_send(HardwareEvent::OutputLevelChanged(bcm_pin_number, new_level));
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let layout_selector = pick_list(
            &Layout::ALL[..],
            Some(self.chosen_layout),
            Message::LayoutChanged,
        )
        .text_size(25)
        .placeholder("Choose Layout");

        let mut main_row = Row::new();

        if let Some(hw_desc) = &self.hardware_description {
            let layout_row = Row::new()
                .push(layout_selector)
                .align_items(Alignment::Center)
                .spacing(10);

            let hardware_desc_row = Row::new()
                .push(hardware_view(hw_desc))
                .align_items(Alignment::Start);

            let file_button_style = CustomButton {
                bg_color: Color::new(0.0, 1.0, 1.0, 1.0),
                text_color: Color::BLACK,
                hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0),
                hovered_text_color: Color::WHITE,
                border_radius: 2.0,
            };

            main_row = main_row.push(
                Column::new()
                    .push(layout_row)
                    .push(hardware_desc_row)
                    .push(
                        Button::new(Text::new("Save Configuration").size(20))
                            .padding(10)
                            .style(file_button_style.get_button_style())
                            .on_press(Message::Save),
                    )
                    .push(
                        Button::new(Text::new("Load Configuration").size(20))
                            .padding(10)
                            .style(file_button_style.get_button_style())
                            .on_press(Message::Load),
                    )
                    .align_items(Alignment::Center)
                    .width(Length::Fixed(400.0))
                    .spacing(10),
            );
        }

        if let Some(pins) = &self.pin_descriptions {
            let pin_layout = match self.chosen_layout {
                Layout::BoardLayout => board_pin_layout_view(pins, &self.gpio_config, self),
                Layout::BCMLayout => bcm_pin_layout_view(pins, &self.gpio_config, self),
            };

            main_row = main_row
                .push(
                    Column::new()
                        .push(pin_layout)
                        .spacing(10)
                        .align_items(Alignment::Center)
                        .height(Length::Fill),
                )
                .align_items(Alignment::Start)
                .width(Length::Fill)
                .height(Length::Fill);
        }

        container(main_row)
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(30)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Top)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.63
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        hw_listener::subscribe().map(Message::HardwareListener)
    }
}

// Hardware Configuration Display
fn hardware_view(hardware_description: &HardwareDescriptor) -> Element<'static, Message> {
    let hardware_info = Column::new()
        .push(Text::new(format!("Hardware: {}", hardware_description.hardware)).size(20))
        .push(Text::new(format!("Revision: {}", hardware_description.revision)).size(20))
        .push(Text::new(format!("Serial: {}", hardware_description.serial)).size(20))
        .push(Text::new(format!("Model: {}", hardware_description.model)).size(20))
        .spacing(10)
        .align_items(Alignment::Center);

    container(hardware_info)
        .padding(10)
        .width(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
}
