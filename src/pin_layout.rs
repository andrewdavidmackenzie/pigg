use std::{env, io};

use iced::widget::{button, container, pick_list, Column, Row, Text};
use iced::{alignment, Alignment, Color, Element, Length, Sandbox, Theme};

// Using Custom Widgets
use crate::custom_widgets::{circle::circle, line::line};

use crate::hw;
use crate::hw::Hardware;
use crate::style::CustomButton;

// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use crate::gpio::{GPIOConfig, PinDescription, PinFunction, GPIO_DESCRIPTION};

pub struct Gpio {
    // TODO this filename will be used when we add a SAVE button or similar
    #[allow(dead_code)]
    config_filename: Option<String>, // filename where to load and save config file to/from
    gpio_description: [PinDescription; 40],
    gpio_config: GPIOConfig,
    pub pin_function_selected: Vec<Option<PinFunction>>,
    clicked: bool,
}

impl Gpio {
    fn get_config(config_filename: Option<String>) -> io::Result<(Option<String>, GPIOConfig)> {
        let gpio_config = match &config_filename {
            None => GPIOConfig::default(),
            Some(filename) => GPIOConfig::load(filename)?,
        };

        Ok((config_filename, gpio_config))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Activate,
    PinFunctionSelected(usize, PinFunction),
}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        // TODO Do this async Elm/Iced style by a Message, that contains a new config to apply
        // Read the (optional) filename of a config to load a config from
        // avoiding the extra overhead of clap or similar while we only have one possible argument
        let (config_filename, gpio_config) =
            Self::get_config(env::args().nth(1)).unwrap_or((None, GPIOConfig::default()));

        // Will need an "Apply" button in the UI to apply config changes to the HW, or apply on each change
        let mut hw = hw::get();
        hw.apply_config(&gpio_config).unwrap(); // TODO handle error

        let num_pins = GPIO_DESCRIPTION.len();
        let pin_function_selected = vec![None; num_pins];

        Self {
            config_filename,
            gpio_description: GPIO_DESCRIPTION,
            gpio_config,
            pin_function_selected,
            clicked: false,
        }
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Activate => self.clicked = true,
            Message::PinFunctionSelected(pin_index, pin_function) => {
                self.pin_function_selected[pin_index] = Some(pin_function);
            } // Message::SubOptionSelected(index, InputPull) => {
              //     self.pin_suboption_selected[index] = Some(InputPull);
              // }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        container(pin_view(&self.gpio_description, &self.gpio_config, self))
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.65
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn get_pin_color(pin_description: &PinDescription) -> CustomButton {
    match pin_description.name {
        "3V3" => CustomButton {
            bg_color: Color::new(1.0, 0.92, 0.016, 1.0), // Yellow
            text_color: Color::BLACK,
        },
        "5V" => CustomButton {
            bg_color: Color::new(1.0, 0.0, 0.0, 1.0), // Red
            text_color: Color::BLACK,
        },
        "Ground" => CustomButton {
            bg_color: Color::BLACK,
            text_color: Color::WHITE,
        },

        "GPIO2" | "GPIO3" => CustomButton {
            bg_color: Color::new(0.678, 0.847, 0.902, 1.0), // Blue
            text_color: Color::WHITE,
        },

        "GPIO7" | "GPIO8" | "GPIO9" | "GPIO10" | "GPIO11" => CustomButton {
            bg_color: Color::new(0.933, 0.510, 0.933, 1.0), // Violet
            text_color: Color::WHITE,
        },

        "GPIO14" | "GPIO15" => CustomButton {
            bg_color: Color::new(0.0, 0.502, 0.0, 1.0), // Green
            text_color: Color::WHITE,
        },

        "ID_SD" | "ID_SC" => CustomButton {
            bg_color: Color::new(0.502, 0.502, 0.502, 1.0), // Grey
            text_color: Color::WHITE,
        },
        _ => CustomButton {
            bg_color: Color::new(1.0, 0.647, 0.0, 1.0), // Orange
            text_color: Color::WHITE,
        },
    }
}

fn pin_view(
    pin_descriptions: &[PinDescription; 40],
    _pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for (idx, pair) in pin_descriptions.chunks(2).enumerate() {
        let mut pin_option_left = Column::new()
            .width(Length::Fixed(140f32))
            .align_items(Alignment::Center);

        if pair[0].options.len() > 1 {
            let mut pin_options_row_left = Row::new().align_items(Alignment::Center);

            pin_options_row_left = pin_options_row_left.push(pick_list(
                pair[0].options,
                gpio.pin_function_selected[idx * 2],
                move |pin_function| Message::PinFunctionSelected(idx * 2, pin_function),
            ));

            pin_option_left = pin_option_left.push(pin_options_row_left);
        }

        let mut pin_name_left = Column::new()
            .width(Length::Fixed(55f32))
            .align_items(Alignment::Center);

        let mut pin_name_left_row = Row::new().align_items(Alignment::Center);
        pin_name_left_row = pin_name_left_row.push(Text::new(pair[0].name));

        pin_name_left = pin_name_left.push(pin_name_left_row);

        let mut pin_name_right = Column::new()
            .width(Length::Fixed(55f32))
            .align_items(Alignment::Center);

        let mut pin_name_right_row = Row::new().align_items(Alignment::Center);
        pin_name_right_row = pin_name_right_row.push(Text::new(pair[1].name));

        pin_name_right = pin_name_right.push(pin_name_right_row);

        let mut pin_arrow_left = Column::new()
            .width(Length::Fixed(60f32))
            .align_items(Alignment::Center);

        let mut pin_arrow_left_row = Row::new().align_items(Alignment::Center);
        pin_arrow_left_row = pin_arrow_left_row.push(circle(5.0));
        pin_arrow_left_row = pin_arrow_left_row.push(line(50.0));

        pin_arrow_left = pin_arrow_left.push(pin_arrow_left_row);

        let mut pin_arrow_right = Column::new()
            .width(Length::Fixed(60f32))
            .align_items(Alignment::Center);

        let mut pin_arrow_right_row = Row::new().align_items(Alignment::Center);
        pin_arrow_right_row = pin_arrow_right_row.push(line(50.0));
        pin_arrow_right_row = pin_arrow_right_row.push(circle(5.0));

        pin_arrow_right = pin_arrow_right.push(pin_arrow_right_row);

        let mut left_pin = Column::new()
            .width(Length::Fixed(40f32))
            .height(Length::Shrink)
            .spacing(10)
            .align_items(Alignment::Center);

        let pin_color_left = get_pin_color(&pair[0]);
        let mut left_pin_row = Row::new().align_items(Alignment::Center);
        left_pin_row = left_pin_row.push(
            button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                .padding(10)
                .width(Length::Fixed(40f32))
                .style(pin_color_left.get_button_style())
                .on_press(Message::Activate),
        );
        left_pin = left_pin.push(left_pin_row);

        let mut right_pin = Column::new()
            .width(Length::Fixed(40f32))
            .height(Length::Shrink)
            .spacing(10)
            .align_items(Alignment::Center);

        let pin_color_right = get_pin_color(&pair[1]);
        let mut right_pin_row = Row::new().align_items(Alignment::Center);
        right_pin_row = right_pin_row.push(
            button(Text::new(pair[1].board_pin_number.to_string()).size(20))
                .padding(10)
                .width(Length::Fixed(40f32))
                .style(pin_color_right.get_button_style())
                .on_press(Message::Activate),
        );
        right_pin = right_pin.push(right_pin_row);

        let mut pin_option_right = Column::new()
            .width(Length::Fixed(140f32))
            .align_items(Alignment::Center);

        if pair[1].options.len() > 1 {
            let mut pin_options_row_right = Row::new().align_items(Alignment::Center);

            pin_options_row_right = pin_options_row_right.push(pick_list(
                pair[1].options,
                gpio.pin_function_selected[idx * 2 + 1],
                move |pin_function| Message::PinFunctionSelected(idx * 2 + 1, pin_function),
            ));

            pin_option_right = pin_option_right.push(pin_options_row_right);
        }

        let row = Row::new()
            .push(pin_option_left)
            .push(pin_name_left)
            .push(pin_arrow_left)
            .push(left_pin)
            .push(right_pin)
            .push(pin_arrow_right)
            .push(pin_name_right)
            .push(pin_option_right)
            .spacing(10)
            .align_items(Alignment::Center);

        column = column.push(row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    container(column).into()
}
