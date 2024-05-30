use rfd::FileDialog;
use std::{env, io};

use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, container, pick_list, Column, Row, Text};
use iced::{
    alignment, executor, window, Alignment, Application, Command, Element, Length, Settings,
    Subscription, Theme,
};

// Custom Widgets
use crate::gpio::{GPIOConfig, PinDescription, PinFunction};
use crate::hw::HardwareDescriptor;
use crate::hw_listener::{HWListenerEvent, HardwareEvent};
// Importing pin layout views
use crate::pin_layout::{logical_pin_view, physical_pin_view};

mod gpio;
mod hw;
mod pin_layout;
mod style;
mod custom_widgets {
    pub mod circle;
    pub mod line;
}
mod hw_listener;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Physical,
    Logical,
}

impl Layout {
    const ALL: [Layout; 2] = [Layout::Physical, Layout::Logical];
}

// Implementing format for Layout
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layout::Physical => "Physical Layout",
                Layout::Logical => "Logical Layout",
            }
        )
    }
}

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        decorations: true,
        size: iced::Size::new(800.0, 900.0),
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    Activate(u8),
    PinFunctionSelected(usize, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
    HardwareListener(HWListenerEvent),
    Save,
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    pub pin_function_selected: [Option<PinFunction>; 40],
    chosen_layout: Layout,
    hardware_description: Option<HardwareDescriptor>,
    listener_sender: Option<Sender<HardwareEvent>>,
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

    // Send the Config from the GUI to the hardware to have it applied
    fn update_hw_config(&mut self) {
        if let Some(ref mut listener) = &mut self.listener_sender {
            let _ = listener.try_send(HardwareEvent::NewConfig(self.gpio_config.clone()));
        }
    }

    // A new function has been selected for a pin via the UI
    // TODO generalize this to ANY change of function
    // TODO doesn't seem to cater for the case where pin is changed back to unconfigured/unused
    fn new_pin_function(&mut self, pin_number: usize, new_function: Option<PinFunction>) {
        let previous_function = self.pin_function_selected[pin_number - 1];
        if new_function != previous_function {
            self.pin_function_selected[pin_number - 1] = new_function;
            if let Some(pins) = &self.pin_descriptions {
                if let Some(bcm_pin_number) = pins[pin_number - 1].bcm_pin_number {
                    // Report config changes to the hardware listener
                    // Since config loading and hardware listener setup can occur out of order
                    // mark the config as changed. If we send to the listener, then mark as done
                    if let Some(ref mut listener) = &mut self.listener_sender {
                        let _ = listener
                            .try_send(HardwareEvent::NewPinConfig(bcm_pin_number, new_function));
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
                pin_function_selected: [None; 40],
                chosen_layout: Layout::Physical,
                hardware_description: None, // Until listener is ready
                listener_sender: None,      // Until listener is ready
                pin_descriptions: None,     // Until listener is ready
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

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::Activate(pin_number) => println!("Pin {pin_number} clicked"),
            Message::PinFunctionSelected(pin_number, pin_function) => {
                // TODO currently there is no way in UI to un-configure a pin!
                self.new_pin_function(pin_number, Some(pin_function));
            }
            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
            }
            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename.clone());
                self.gpio_config = config.clone();

                for (pin_number, pin_function) in config.configured_pins.iter() {
                    self.pin_function_selected[*pin_number as usize - 1] = Some(*pin_function);
                }

                self.update_hw_config();
            }
            Message::None => {}
            Message::Save => {

                // Synchronous file handling
                if let Some(path) = FileDialog::new().set_title("Choose files").save_file() {
                    let path_str = path.to_str().unwrap();

                    if let Err(err) = self.gpio_config.save(path_str) {
                        eprintln!("Error saving configuration to {}: {}", path.display(), err);
                    } else {
                        println!("Configuration saved to {}", path.display());
                    }
                    println!("{:?}", self.gpio_config);
                } else {
                    println!("No file selected for saving configuration.");
                }
            }

            Message::HardwareListener(event) => match event {
                HWListenerEvent::Ready(config_change_sender, hw_desc, pins) => {
                    self.listener_sender = Some(config_change_sender);
                    self.hardware_description = Some(hw_desc);
                    self.pin_descriptions = Some(pins);
                    self.update_hw_config();
                }
                HWListenerEvent::InputChange(level_change) => {
                    println!("Input changed: {:?}", level_change);
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

            main_row = main_row.push(
                Column::new()
                    .push(layout_row)
                    .push(hardware_desc_row)
                    .push(button("save config").on_press(Message::Save))
                    .align_items(Alignment::Center)
                    .width(Length::Fixed(400.0))
                    .spacing(10),
            );
        }

        if let Some(pins) = &self.pin_descriptions {
            let pin_layout = match self.chosen_layout {
                Layout::Physical => physical_pin_view(pins, &self.gpio_config, self),
                Layout::Logical => logical_pin_view(pins, &self.gpio_config, self),
            };

            main_row = main_row
                .push(
                    Column::new()
                        .push(pin_layout)
                        .spacing(10)
                        .align_items(Alignment::Center)
                        .width(Length::Fixed(700.0))
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
