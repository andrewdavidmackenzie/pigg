use std::{env, io};

use iced::widget::{container, pick_list, Column, Row, Text};
use iced::{
    alignment, executor, window, Alignment, Application, Command, Element, Length, Settings, Theme,
};

// Custom Widgets
use crate::gpio::{GPIOConfig, PinFunction};
use crate::hw::Hardware;
use crate::hw::HardwareDescriptor;

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

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    connected_hardware: Box<dyn Hardware>,
    pub pin_function_selected: Vec<Option<PinFunction>>,
    clicked: bool,
    chosen_layout: Layout,
    hardware_description: HardwareDescriptor,
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

#[derive(Debug, Clone)]
pub enum Message {
    Activate,
    PinFunctionSelected(usize, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
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
                pin_function_selected,
                clicked: false,
                chosen_layout: Layout::Physical,
                connected_hardware: Box::new(hw),
                hardware_description,
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
            Message::PinFunctionSelected(pin_index, pin_function) => {
                self.pin_function_selected[pin_index] = Some(pin_function);
            }
            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
            }
            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename);
                self.connected_hardware.apply_config(&config).unwrap();
                // TODO refresh the UI as a new config was loaded
            }
            Message::None => {}
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
}

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
