mod gpio;
mod hw;
mod custom_widgets {
    pub mod circle;
    pub mod line;
}

use std::env;
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use crate::gpio::{GPIOConfig, PinDescription, GPIO_DESCRIPTION};
// Using Custom Widgets
use custom_widgets::{circle::circle, line::line};
use iced::widget::{button, container, Column, Row, Text};
use iced::{alignment, window, Alignment, Color, Element, Length, Sandbox, Settings, Theme};

// Use Hardware via trait
//use hw::Hardware;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        decorations: true,
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
        // TODO factor this out into a function, once the UI update, async and error handling
        // is done.
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
                    }
                    _ => {
                        // TODO put this on the UI in some way
                        println!("Failed to load GPIO Config from file: {filename}");
                        println!("Default GPIO Config will be used instead");
                        GPIOConfig::default()
                    }
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
        0.65
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn pin_view(
    pin_descriptions: &[PinDescription; 40],
    _pin_config: &GPIOConfig,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pair in pin_descriptions.chunks(2) {
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

        match pair[0].board_pin_number {
            1 | 17 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(1.0, 0.92, 0.016, 1.0),
                    text_color: Color::BLACK,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }
            3 | 5 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.678, 0.847, 0.902, 1.0),
                    text_color: Color::BLACK,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }

            19 | 21 | 23 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.933, 0.510, 0.933, 1.0),
                    text_color: Color::WHITE,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }

            27 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.502, 0.502, 0.502, 1.0),
                    text_color: Color::WHITE,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }

            9 | 25 | 39 => {
                let pin_color = CustomButton {
                    bg_color: Color::BLACK,
                    text_color: Color::WHITE,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }

            _ => {
                let pin_color = CustomButton {
                    bg_color: Color::new(1.0, 0.647, 0.0, 1.0),
                    text_color: Color::WHITE,
                };
                let mut left_pin_row = Row::new().align_items(Alignment::Center);
                left_pin_row = left_pin_row.push(
                    button(Text::new(pair[0].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                left_pin = left_pin.push(left_pin_row);
            }
        }

        let mut right_pin = Column::new()
            .width(Length::Fixed(40f32))
            .height(Length::Shrink)
            .spacing(10)
            .align_items(Alignment::Center);

        match pair[1].board_pin_number {
            2 | 4 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(1.0, 0.0, 0.0, 1.0),
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    iced::widget::Button::new(
                        Text::new(pair[1].board_pin_number.to_string()).size(20),
                    )
                    .padding(10)
                    .width(Length::Fixed(40f32))
                    .style(pin_color.get_button_style())
                    .on_press(Message::Activate),
                );

                right_pin = right_pin.push(right_pin_row);
            }

            6 | 14 | 20 | 30 | 34 => {
                let pin_color = CustomButton {
                    bg_color: Color::BLACK,
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    iced::widget::Button::new(
                        Text::new(pair[1].board_pin_number.to_string()).size(20),
                    )
                    .padding(10)
                    .width(Length::Fixed(40f32))
                    .style(pin_color.get_button_style())
                    .on_press(Message::Activate),
                );

                right_pin = right_pin.push(right_pin_row);
            }

            8 | 10 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.0, 0.502, 0.0, 1.0),
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    iced::widget::Button::new(
                        Text::new(pair[1].board_pin_number.to_string()).size(20),
                    )
                    .padding(10)
                    .width(Length::Fixed(40f32))
                    .style(pin_color.get_button_style())
                    .on_press(Message::Activate),
                );

                right_pin = right_pin.push(right_pin_row);
            }
            24 | 26 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.933, 0.510, 0.933, 1.0),
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    button(Text::new(pair[1].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                right_pin = right_pin.push(right_pin_row);
            }

            28 => {
                let pin_color = CustomButton {
                    bg_color: Color::new(0.502, 0.502, 0.502, 1.0),
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    button(Text::new(pair[1].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                right_pin = right_pin.push(right_pin_row);
            }

            _ => {
                let pin_color = CustomButton {
                    bg_color: Color::new(1.0, 0.647, 0.0, 1.0),
                    text_color: Color::WHITE,
                };
                let mut right_pin_row = Row::new().align_items(Alignment::Center);
                right_pin_row = right_pin_row.push(
                    button(Text::new(pair[1].board_pin_number.to_string()).size(20))
                        .padding(10)
                        .width(Length::Fixed(40f32))
                        .style(pin_color.get_button_style())
                        .on_press(Message::Activate),
                );
                right_pin = right_pin.push(right_pin_row);
            }
        }
        let row = Row::new()
            .push(pin_name_left)
            .push(pin_arrow_left)
            .push(left_pin)
            .push(right_pin)
            .push(pin_arrow_right)
            .push(pin_name_right)
            .spacing(10)
            .align_items(Alignment::Center);

        column = column.push(row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    container(column).into()
}

pub struct CustomButton {
    bg_color: iced::Color,
    text_color: iced::Color,
}

impl button::StyleSheet for CustomButton {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: Some(iced::Background::Color(self.bg_color)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 50.0.into(),
            },
            text_color: self.text_color,
            ..Default::default()
        }
    }
}

impl CustomButton {
    pub fn get_button_style(&self) -> iced::widget::theme::Button {
        iced::widget::theme::Button::Custom(Box::new(CustomButton {
            bg_color: self.bg_color,
            text_color: self.text_color,
        }))
    }
}
