mod gpio;
mod hw;
mod custom_widgets {
    pub mod circle;
    pub mod line;
}

use std::env;
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use crate::gpio::{PinDescription, GPIO_DESCRIPTION, GPIOConfig};
// Using Custom Widgets
use custom_widgets::{circle::circle, line::line};
use iced::widget::{button, container, Column, Row, Text};
use iced::{alignment, window, Alignment, Element, Length, Sandbox, Settings};
// Use Hardware via trait
//use hw::Hardware;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    // Will need an "Apply" button in the UI to apply config changes to the HW, or apply on each change
    //let mut hw = hw::get();
    //hw.apply_config(&config);

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

struct Gpio {
    // TODO this filename will be used when we add a SAVE button or similar
    #[allow(dead_code)]
    config_file: Option<String>, // filename where to load and save config file to/from
    gpio_description: [PinDescription; 40],
    gpio_config: GPIOConfig,
    clicked: bool,
}

#[derive(Debug, Clone, Copy)]

enum Message {
    Activate,
}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        // filename of config to load is an optional command line argument to piggui
        // avoiding the extra overhead of clap or similar while we only have one possible argument
        let config_file = env::args().nth(1);
        let gpio_config = match &config_file {
            None => GPIOConfig::default(),
            Some(filename) => {
                // TODO maybe do asynchronously, and send a message with the config when loaded?
                match GPIOConfig::load(filename) {
                    Ok(config) => {
                        // TODO put this on the UI in some way
                        println!("GPIO Config loaded from file: {filename}");
                        config
                    },
                    _ => GPIOConfig::default()
                }
            }
        };

        Self {
            config_file,
            gpio_description: GPIO_DESCRIPTION,
            gpio_config,
            clicked: false,
        }
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Activate => self.clicked = true,
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        container(pin_view(&self.gpio_description, &self.gpio_config))
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.54
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn pin_view(pin_descriptions: &[PinDescription; 40],
            _pin_config: &GPIOConfig) -> Element<'static, Message> {
    // TODO: Align Layout
    let mut column = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    for pair in pin_descriptions.chunks(2) {
        let mut row = Row::new()
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center);

        row = row.push(Text::new(pair[0].name).size(20));

        let mut r1 = Row::new().align_items(Alignment::Center);
        r1 = r1.push(circle(5.0));
        r1 = r1.push(line(50.0));
        row = row.push(r1);

        row = row.push(
            button(Text::new(pair[0].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed(50f32)),
        );
        row = row.push(
            button(Text::new(pair[1].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed(50f32)),
        );
        let mut r2 = Row::new().align_items(Alignment::Center);
        r2 = r2.push(line(50.0));
        r2 = r2.push(circle(5.0));
        row = row.push(r2);

        row = row.push(Text::new(pair[1].name).size(20));

        column = column.push(row);
    }

    container(column).into()
}
