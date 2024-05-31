use iced::{Alignment, Color, Element, Length};
use iced::widget::{button, Column, container, pick_list, Row, Text};

use crate::custom_widgets::{circle::circle, line::line};
use crate::gpio::{GPIOConfig, PinDescription, PinFunction};
use crate::Gpio;
use crate::Message;
use crate::style::CustomButton;

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

/// View that lays out the pins in a single column ordered by BCM pin number
pub fn bcm_pin_layout_view(
    pin_descriptions: &[PinDescription; 40],
    pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    let mut gpio_pins = pin_descriptions
        .iter()
        .filter(|pin| pin.options.len() > 1)
        .filter(|pin| pin.bcm_pin_number.is_some())
        .collect::<Vec<&PinDescription>>();
    let pins_slice = gpio_pins.as_mut_slice();
    pins_slice.sort_by_key(|pin| pin.bcm_pin_number.unwrap());

    for pin in pins_slice {
        let pin_row = create_pin_view_side(pin, select_pin_function(pin, pin_config, &gpio), true);

        column = column.push(pin_row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    container(column).into()
}

/// View that draws the pins laid out as they are on the physical Pi board
/// View that draws the pins laid out as they are on the physical Pi board
pub fn board_pin_layout_view(
    pin_descriptions: &[PinDescription; 40],
    pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pair in pin_descriptions.chunks(2) {
        let left_row = create_pin_view_side(
            &pair[0],
            select_pin_function(&pair[0], pin_config, gpio),
            true,
        );

        let right_row = create_pin_view_side(
            &pair[1],
            select_pin_function(&pair[1], pin_config, gpio),
            false,
        );

        let row = Row::new()
            .push(left_row)
            .push(right_row)
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
    is_left: bool,
) -> Row<'static, Message> {
    let mut pin_option = Column::new()
        .width(Length::Fixed(140f32))
        .align_items(Alignment::Center);
    let pin_number = pin.board_pin_number as usize;

    if pin.options.len() > 1 {
        let mut pin_options_row = Row::new()
            .align_items(Alignment::Center)
            .width(Length::Fixed(140f32));

        pin_options_row = pin_options_row.push(
            pick_list(pin.options, selected_function, move |pin_function| {
                Message::PinFunctionSelected(pin_number, pin_function)
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

    let pin_color = get_pin_color(pin);
    let mut pin_button_row = Row::new().align_items(Alignment::Center);
    pin_button_row = pin_button_row.push(
        button(Text::new(pin.board_pin_number.to_string()).size(20))
            .padding(10)
            .width(Length::Fixed(40f32))
            .style(pin_color.get_button_style())
            .on_press(Message::Activate(pin.board_pin_number)),
    );
    pin_button = pin_button.push(pin_button_row);

    if is_left {
        Row::new()
            .push(pin_option)
            .push(pin_name)
            .push(pin_arrow)
            .push(pin_button)
            .spacing(10)
            .align_items(Alignment::Center)
    } else {
        Row::new()
            .push(pin_button)
            .push(pin_arrow)
            .push(pin_name)
            .push(pin_option)
            .spacing(10)
            .align_items(Alignment::Center)
    }
}

fn select_pin_function(
    pin: &PinDescription,
    pin_config: &GPIOConfig,
    gpio: &Gpio,
) -> Option<PinFunction> {
    pin_config
        .configured_pins
        .iter()
        .find_map(|(pin_number, pin_function)| {
            // Check if the pin has a BCM number
            if let Some(bcm_pin_number) = pin.bcm_pin_number {
                // If the pin number matches the BCM number, use the configured pin function
                if *pin_number == bcm_pin_number {
                    Some(*pin_function)
                } else {
                    None
                }
            } else {
                None
            }
        })
        // Else Select from the UI
        .or_else(|| gpio.pin_function_selected[pin.board_pin_number as usize - 1])
}
