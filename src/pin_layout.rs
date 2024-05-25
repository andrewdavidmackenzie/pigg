use std::{env, io};

use iced::{alignment, Alignment, Color, Element, Length, Sandbox, Theme};
use iced::widget::{button, Column, container, pick_list, Row, Text};

// Using Custom Widgets
use crate::custom_widgets::{circle::circle, line::line};
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use crate::gpio::{GPIO_DESCRIPTION, GPIOConfig, PinDescription, PinFunction};
use crate::hw;
use crate::hw::Hardware;
use crate::style::CustomButton;

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

pub struct Gpio {
    // TODO this filename will be used when we add a SAVE button or similar
    #[allow(dead_code)]
    config_filename: Option<String>, // filename where to load and save config file to/from
    gpio_description: [PinDescription; 40],
    gpio_config: GPIOConfig,
    pub pin_function_selected: Vec<Option<PinFunction>>,
    clicked: bool,
    chosen_layout: Layout,
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
    LayoutChanged(Layout),
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
            chosen_layout: Layout::Physical,
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
            }
            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let layout_selector = pick_list(
            &Layout::ALL[..],
            Some(self.chosen_layout),
            Message::LayoutChanged,
        )
        .placeholder("Choose Layout");

        let pin_layout = match self.chosen_layout {
            Layout::Physical => physical_pin_view(&self.gpio_description, &self.gpio_config, self),
            Layout::Logical => logical_pin_view(&self.gpio_description, &self.gpio_config, self),
        };

        let main_column = Column::new()
            .push(
                Column::new()
                    .push(layout_selector)
                    .align_items(Alignment::Center)
                    .width(Length::Fill)
                    .padding(10),
            )
            .push(iced::widget::Space::new(
                Length::Fixed(1.0),
                Length::Fixed(20.0),
            ))
            .push(
                Column::new()
                    .push(pin_layout)
                    .align_items(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start);

        container(main_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Top)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.68
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

// Logical view layout
fn logical_pin_view(
    pin_descriptions: &[PinDescription; 40],
    _pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pin in pin_descriptions
        .iter()
        .filter(|pin| pin.options.len() > 1)
        .filter(|pin| pin.bcm_pin_number.is_some())
    {
        let (pin_option, pin_name, pin_arrow, pin_button) = create_pin_view_side(
            pin,
            gpio.pin_function_selected[pin.board_pin_number as usize - 1],
            pin.board_pin_number as usize,
            true,
        );

        let pin_row = Row::new()
            .push(pin_option)
            .push(pin_button)
            .push(pin_arrow)
            .push(pin_name)
            .spacing(10)
            .align_items(Alignment::Center);

        column = column.push(pin_row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    container(column).into()
}

// Physical pin layout
fn physical_pin_view(
    pin_descriptions: &[PinDescription; 40],
    _pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pair in pin_descriptions.chunks(2) {
        let left_view = create_pin_view_side(
            &pair[0],
            gpio.pin_function_selected[pair[0].board_pin_number as usize - 1],
            pair[0].board_pin_number as usize,
            true,
        );

        let right_view = create_pin_view_side(
            &pair[1],
            gpio.pin_function_selected[pair[1].board_pin_number as usize - 1],
            pair[1].board_pin_number as usize,
            false,
        );

        let row = Row::new()
            .push(left_view.0)
            .push(left_view.1)
            .push(left_view.2)
            .push(left_view.3)
            .push(right_view.3)
            .push(right_view.2)
            .push(right_view.1)
            .push(right_view.0)
            .spacing(10)
            .align_items(Alignment::Center);

        column = column.push(row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    container(column).into()
}

fn create_pin_view_side(
    pin: &PinDescription,
    selected_function: Option<PinFunction>,
    idx: usize,
    is_left: bool,
) -> (
    Column<'static, Message>,
    Column<'static, Message>,
    Column<'static, Message>,
    Column<'static, Message>,
) {
    let mut pin_option = Column::new()
        .width(Length::Fixed(140f32))
        .align_items(Alignment::Center);

    if pin.options.len() > 1 {
        let mut pin_options_row = Row::new()
            .align_items(Alignment::Center)
            .width(Length::Fixed(140f32));

        pin_options_row = pin_options_row.push(
            pick_list(pin.options, selected_function, move |pin_function| {
                Message::PinFunctionSelected(idx, pin_function)
            })
            .placeholder("Select function"),
        );

        pin_option = pin_option.push(pin_options_row);
    }

    let mut pin_name = Column::new()
        .width(Length::Fixed(55f32))
        .align_items(Alignment::Center);

    let mut pin_name_row = Row::new().align_items(Alignment::Center);
    pin_name_row = pin_name_row.push(Text::new(pin.name));

    pin_name = pin_name.push(pin_name_row);

    let mut pin_arrow = Column::new()
        .width(Length::Fixed(60f32))
        .align_items(Alignment::Center);

    let mut pin_arrow_row = Row::new().align_items(Alignment::Center);
    if is_left {
        pin_arrow_row = pin_arrow_row.push(circle(5.0));
        pin_arrow_row = pin_arrow_row.push(line(50.0));
    } else {
        pin_arrow_row = pin_arrow_row.push(line(50.0));
        pin_arrow_row = pin_arrow_row.push(circle(5.0));
    }

    pin_arrow = pin_arrow.push(pin_arrow_row);

    let mut pin_button = Column::new()
        .width(Length::Fixed(40f32))
        .height(Length::Shrink)
        .spacing(10)
        .align_items(Alignment::Center);

    let pin_color = get_pin_color(&pin);
    let mut pin_button_row = Row::new().align_items(Alignment::Center);
    pin_button_row = pin_button_row.push(
        button(Text::new(pin.board_pin_number.to_string()).size(20))
            .padding(10)
            .width(Length::Fixed(40f32))
            .style(pin_color.get_button_style())
            .on_press(Message::Activate),
    );
    pin_button = pin_button.push(pin_button_row);

    (pin_option, pin_name, pin_arrow, pin_button)
}
