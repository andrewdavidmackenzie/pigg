use iced::{Alignment, Color, Element, Length};
use iced::widget::{button, Column, container, pick_list, Row, Text, toggler};

use crate::custom_widgets::{circle::circle, line::line};
use crate::custom_widgets::led::led;
use crate::gpio::{
    BCMPinNumber, BoardPinNumber, GPIOConfig, PinDescription, PinFunction, PinLevel,
};
use crate::Gpio;
use crate::gpio::PinFunction::Input;
use crate::InputPull;
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
        let pin_row = create_pin_view_side(
            pin,
            select_pin_function(pin, pin_config, gpio),
            true,
            &gpio.pin_function_selected[pin.board_pin_number as usize - 1],
            &gpio.pin_states,
            pin_config,
        );

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
            &gpio.pin_function_selected[pair[0].board_pin_number as usize - 1],
            &gpio.pin_states,
            pin_config,
        );

        let right_row = create_pin_view_side(
            &pair[1],
            select_pin_function(&pair[1], pin_config, gpio),
            false,
            &gpio.pin_function_selected[pair[1].board_pin_number as usize - 1],
            &gpio.pin_states,
            pin_config,
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

/// Create the widget that either shows an input pin's state,
/// or allows the user to control the state of an output pin
fn get_pin_widget(
    board_pin_number: BoardPinNumber,
    bcm_pin_number: Option<BCMPinNumber>,
    pin_function: &Option<PinFunction>,
    pin_state: Option<PinLevel>,
    _pin_config: &GPIOConfig,
    is_left: bool,
) -> Row<'static, Message> {
    let row = match pin_function {
        Some(PinFunction::Input(pull)) => {
            let mut sub_options = vec![InputPull::PullUp, InputPull::PullDown, InputPull::None];

            // Filter out the currently selected pull option
            if let Some(selected_pull) = pull {
                sub_options.retain(|&option| &option != selected_pull);
            }

            if is_left {
                Row::new()
                    .push(led(12.0, pin_state))
                    .push(
                        pick_list(sub_options, *pull, move |selected_pull| {
                            Message::PinFunctionSelected(
                                board_pin_number,
                                bcm_pin_number.unwrap(),
                                Input(Some(selected_pull)),
                            )
                        })
                        .placeholder("Select Input"),
                    )
                    .spacing(10)
            } else {
                Row::new()
                    .push(
                        pick_list(sub_options, *pull, move |selected_pull| {
                            Message::PinFunctionSelected(
                                board_pin_number,
                                bcm_pin_number.unwrap(),
                                Input(Some(selected_pull)),
                            )
                        })
                        .placeholder("Select Input"),
                    )
                    .push(led(12.0, pin_state))
                    .spacing(10)
            }
        }

        // TODO Fix Output Width
        Some(PinFunction::Output(_)) => {
            let toggler = toggler(None, pin_state.unwrap_or(false), move |b| {
                Message::ChangeOutputLevel(bcm_pin_number.unwrap(), b)
            });
            Row::new().push(toggler)
        }
        _ => Row::new(),
    };
    row.width(Length::Fixed(150f32))
        .align_items(Alignment::Center)
}

/// Create a row of widgets that represent a pin, either from left to right or right to left
fn create_pin_view_side(
    pin: &PinDescription,
    selected_function: Option<PinFunction>,
    is_left: bool,
    pin_function: &Option<PinFunction>,
    pin_states: &[Option<PinLevel>; 40],
    pin_config: &GPIOConfig,
) -> Row<'static, Message> {
    // Create a widget that is either used to visualize an input or control an output
    let pin_state = match pin.bcm_pin_number {
        None => None,
        Some(bcm) => pin_states[bcm as usize],
    };
    let pin_widget = get_pin_widget(
        pin.board_pin_number,
        pin.bcm_pin_number,
        pin_function,
        pin_state,
        pin_config,
        is_left,
    );

    // Create the drop-down selector of pin function
    let mut pin_option = Column::new()
        .width(Length::Fixed(140f32))
        .align_items(Alignment::Center);
    if pin.options.len() > 1 {
        let board_pin_number = pin.board_pin_number;
        let bcm_pin_number = pin.bcm_pin_number.unwrap();
        let mut pin_options_row = Row::new()
            .align_items(Alignment::Center)
            .width(Length::Fixed(140f32));
        pin_options_row = pin_options_row.push(
            pick_list(pin.options, selected_function, move |pin_function| {
                Message::PinFunctionSelected(board_pin_number, bcm_pin_number, pin_function)
            })
            .width(Length::Fixed(200f32))
            .placeholder("Select function"),
        );

        pin_option = pin_option.push(pin_options_row);
    }

    // Create the Pin name
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

    // Create the "pin arrow" a small drawing to illustrate the pin
    pin_arrow = pin_arrow.push(pin_arrow_row);

    // Create the pin itself, with number and as a button
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

    // Create the row of widgets that represent the pin, inverted order if left or right
    if is_left {
        Row::new()
            .push(pin_widget)
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
            .push(pin_widget)
            .spacing(10)
            .align_items(Alignment::Center)
    }
}

pub(crate) fn select_pin_function(
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
