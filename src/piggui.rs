use std::{env, io};

use iced::{
    alignment, Alignment, Application, Command, Element, executor, Length, Settings, Subscription,
    Theme, window,
};
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::widget::{Column, container, pick_list, Row, Text};

// Custom Widgets
use crate::gpio::{GPIOConfig, PinFunction};
use crate::hw::Hardware;
use crate::hw::HardwareDescriptor;
use crate::hw_listener::{ConfigEvent, ListenerEvent};
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
    Activate,
    PinFunctionSelected(usize, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
    HardwareListener(ListenerEvent),
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    config_changed: bool,
    connected_hardware: Box<dyn Hardware>,
    pub pin_function_selected: Vec<Option<PinFunction>>,
    clicked: bool,
    chosen_layout: Layout,
    hardware_description: HardwareDescriptor,
    listener_sender: Option<Sender<ConfigEvent>>,
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
}

impl Application for Gpio {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Gpio, Command<Self::Message>) {
        let hw = hw::get();
        let num_pins = hw.pin_descriptions().len();
        let pin_function_selected = vec![None; num_pins];
        let hardware_description = hw.descriptor().unwrap();

        (
            Self {
                config_filename: None,
                gpio_config: GPIOConfig::default(),
                config_changed: false,
                pin_function_selected,
                clicked: false,
                chosen_layout: Layout::Physical,
                connected_hardware: Box::new(hw),
                hardware_description,
                listener_sender: None,
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
            Message::Activate => self.clicked = true,
            Message::PinFunctionSelected(pin_number, pin_function) => {
                let previous_function = self.pin_function_selected[pin_number - 1];
                self.pin_function_selected[pin_number - 1] = Some(pin_function);
                if let Some(bcm_pin_number) =
                    self.connected_hardware.pin_descriptions()[pin_number - 1].bcm_pin_number
                {
                    // TODO error reporting if config cannot be applied
                    let _ = self
                        .connected_hardware
                        .apply_pin_config(bcm_pin_number, &pin_function);
                    self.config_changed = true;

                    // Report config changes to the hardware listener
                    // Since config loading and hardware listener setup can occur out of order
                    // mark the config as changed. If we send to the listener, then mark as done
                    match (previous_function, pin_function) {
                        (Some(PinFunction::Input(_)), PinFunction::Input(_)) => { /* No change */ }
                        (Some(PinFunction::Input(_)), _) => {
                            // was an input, not anymore
                            if let Some(ref mut listener) = &mut self.listener_sender {
                                println!("Informing listener of InputPin removal");
                                let _ = listener.send(ConfigEvent::InputPinRemoved(bcm_pin_number));
                                self.config_changed = false;
                            }
                        }
                        (_, PinFunction::Input(_)) => {
                            // was not an input, is now
                            if let Some(ref mut listener) = &mut self.listener_sender {
                                println!("Informing listener of InputPin addition");
                                let _ = listener.send(ConfigEvent::InputPinAdded(bcm_pin_number));
                                self.config_changed = false;
                            }
                        }
                        (_, _) => { /* Don't care! */ }
                    }
                }
            }
            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
            }
            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename);
                // TODO error reporting if config cannot be applied
                self.connected_hardware.apply_config(&config).unwrap();
                self.gpio_config = config.clone();
                self.config_changed = true;
                // TODO refresh the UI as a new config was loaded

                // Since config loading and hardware listener setup can occur out of order
                // track if there is already a hw_listener that needs to get this config change
                if let Some(ref mut listener) = &mut self.listener_sender {
                    println!("Informing listener of config change");
                    let _ = listener.send(ConfigEvent::HardwareConfigured(
                        config,
                        Box::new(self.connected_hardware.pin_descriptions()),
                    ));
                    self.config_changed = false;
                }
            }
            Message::None => {}
            Message::HardwareListener(event) => match event {
                ListenerEvent::Ready(config_change_sender) => {
                    println!("GUI got listener sender to use on config changes");
                    self.listener_sender = Some(config_change_sender);
                    // Since config loading and hardware listener setup can occur out of order
                    // track if there has been a config change made that is pending to send to
                    // the hw_listener, and if so, send it
                    if self.config_changed {
                        if let Some(ref mut listener) = &mut self.listener_sender {
                            println!("Informing listener of config change");
                            let _ = listener.send(ConfigEvent::HardwareConfigured(
                                self.gpio_config.clone(),
                                Box::new(self.connected_hardware.pin_descriptions()),
                            ));
                            self.config_changed = false;
                        }
                    }
                }
                ListenerEvent::InputChange(level_change) => {
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

        let pin_layout = match self.chosen_layout {
            Layout::Physical => physical_pin_view(
                &self.connected_hardware.pin_descriptions(),
                &self.gpio_config,
                self,
            ),
            Layout::Logical => logical_pin_view(
                &self.connected_hardware.pin_descriptions(),
                &self.gpio_config,
                self,
            ),
        };
        let layout_row = Row::new()
            .push(layout_selector)
            .align_items(Alignment::Center)
            .spacing(10);

        let hardware_desc_row = Row::new()
            .push(hardware_view(&self.hardware_description))
            .align_items(Alignment::Start);

        let main_column = Row::new()
            .push(
                Column::new()
                    .push(layout_row)
                    .push(hardware_desc_row)
                    .align_items(Alignment::Center)
                    .width(Length::Fixed(400.0))
                    .spacing(10),
            )
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

        container(main_column)
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
