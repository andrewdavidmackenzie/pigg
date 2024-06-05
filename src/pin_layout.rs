use iced::{Alignment, Color, Element, Length};
use iced::alignment::Horizontal;
use iced::widget::{button, Column, pick_list, Row, Text, toggler};

use crate::{Gpio, PinState};
use crate::custom_widgets::{circle::circle, led::led, line::line};
use crate::hw::{
    BCMPinNumber, BoardPinNumber, LevelChange, PinDescription, PinDescriptionSet, PinFunction,
    PinLevel,
};
use crate::hw::PinFunction::{Input, Output};
use crate::InputPull;
use crate::Message;
use crate::style::CustomButton;

fn get_pin_color(pin_description: &PinDescription) -> CustomButton {
    match pin_description.name {
        "3V3" => CustomButton {
            bg_color: Color::new(1.0, 0.92, 0.016, 1.0), // Yellow
            text_color: Color::BLACK,
            border_radius: 50.0,
            hovered_bg_color: Color::new(1.0, 1.0, 0.0, 1.0),
            hovered_text_color: Color::BLACK,
        },
        "5V" => CustomButton {
            bg_color: Color::new(1.0, 0.0, 0.0, 1.0), // Red
            text_color: Color::BLACK,
            border_radius: 50.0,
            hovered_bg_color: Color::new(1.0, 0.0, 0.0, 1.0),
            hovered_text_color: Color::BLACK,
        },
        "Ground" => CustomButton {
            bg_color: Color::BLACK,
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::BLACK,
        },

        "GPIO2" | "GPIO3" => CustomButton {
            bg_color: Color::new(0.678, 0.847, 0.902, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.678, 0.847, 0.902, 1.0),
        },

        "GPIO7" | "GPIO8" | "GPIO9" | "GPIO10" | "GPIO11" => CustomButton {
            bg_color: Color::new(0.933, 0.510, 0.933, 1.0), // Violet
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.933, 0.510, 0.933, 1.0),
        },

        "GPIO14" | "GPIO15" => CustomButton {
            bg_color: Color::new(0.0, 0.502, 0.0, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.0, 0.502, 0.0, 1.0),
        },

        "ID_SD" | "ID_SC" => CustomButton {
            bg_color: Color::new(0.502, 0.502, 0.502, 1.0), // Grey
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.502, 0.502, 0.502, 1.0),
        },
        _ => CustomButton {
            bg_color: Color::new(1.0, 0.647, 0.0, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(1.0, 0.647, 0.0, 1.0),
        },
    }
}

/// View that lays out the pins in a single column ordered by BCM pin number
pub fn bcm_pin_layout_view(pin_set: &PinDescriptionSet, gpio: &Gpio) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pin in pin_set.bcm_pins_sorted() {
        let pin_row = create_pin_view_side(
            pin,
            gpio.pin_function_selected[pin.board_pin_number as usize - 1],
            true,
            &gpio.pin_states[pin.board_pin_number as usize - 1],
        );

        column = column.push(pin_row).push(iced::widget::Space::new(
            Length::Fixed(1.0),
            Length::Fixed(5.0),
        ));
    }

    column.into()
}

/// View that draws the pins laid out as they are on the physical Pi board
/// View that draws the pins laid out as they are on the physical Pi board
pub fn board_pin_layout_view(
    pin_descriptions: &PinDescriptionSet,
    gpio: &Gpio,
) -> Element<'static, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    // Draw all pins, those with and without BCM pin numbers
    for pair in pin_descriptions.pins().chunks(2) {
        let left_row = create_pin_view_side(
            &pair[0],
            gpio.pin_function_selected[pair[0].board_pin_number as usize - 1],
            true,
            &gpio.pin_states[pair[0].board_pin_number as usize - 1],
        );

        let right_row = create_pin_view_side(
            &pair[1],
            gpio.pin_function_selected[pair[1].board_pin_number as usize - 1],
            false,
            &gpio.pin_states[pair[1].board_pin_number as usize - 1],
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

    column.into()
}

/// Create the widget that either shows an input pin's state,
/// or allows the user to control the state of an output pin
/// This should only be called for pins that have a valid BCMPinNumber
fn get_pin_widget(
    board_pin_number: BoardPinNumber,
    bcm_pin_number: BCMPinNumber,
    pin_function: &PinFunction,
    pin_state: &PinState,
    is_left: bool,
) -> Row<'static, Message> {
    let row = match pin_function {
        Input(pull) => {
            let mut sub_options = vec![InputPull::PullUp, InputPull::PullDown, InputPull::None];

            // Filter out the currently selected pull option
            if let Some(selected_pull) = pull {
                sub_options.retain(|&option| &option != selected_pull);
            }

            if is_left {
                Row::new().push(led(12.0, pin_state.get_level())).push(
                    pick_list(sub_options, *pull, move |selected_pull| {
                        Message::PinFunctionSelected(
                            board_pin_number,
                            bcm_pin_number,
                            Input(Some(selected_pull)),
                        )
                    })
                    .placeholder("Select Pullup"),
                )
            } else {
                Row::new()
                    .push(
                        pick_list(sub_options, *pull, move |selected_pull| {
                            Message::PinFunctionSelected(
                                board_pin_number,
                                bcm_pin_number,
                                Input(Some(selected_pull)),
                            )
                        })
                        .placeholder("Select Pullup"),
                    )
                    .push(led(12.0, pin_state.get_level()))
            }
        }

        Output(_) => {
            let toggler = toggler(
                None,
                pin_state.get_level().unwrap_or(false as PinLevel),
                move |b| Message::ChangeOutputLevel(LevelChange::new(bcm_pin_number, b)),
            );
            if is_left {
                Row::new().push(Column::new().push(toggler).align_items(Alignment::End))
            } else {
                Row::new().push(Column::new().push(toggler).align_items(Alignment::Start))
            }
        }

        _ => Row::new(),
    };
    row.width(Length::Fixed(150f32))
        .spacing(10)
        .align_items(Alignment::Center)
}

/// Create a row of widgets that represent a pin, either from left to right or right to left
fn create_pin_view_side(
    pin_description: &PinDescription,
    selected_function: PinFunction,
    is_left: bool,
    pin_state: &PinState,
) -> Row<'static, Message> {
    // Create a widget that is either used to visualize an input or control an output
    let pin_widget = match pin_description.bcm_pin_number {
        None => Row::new(),
        Some(bcm) => get_pin_widget(
            pin_description.board_pin_number,
            bcm,
            &selected_function,
            pin_state,
            is_left,
        ),
    };

    // Create the drop-down selector of pin function
    let mut pin_option = Column::new()
        .width(Length::Fixed(140f32))
        .align_items(Alignment::Center);
    if pin_description.options.len() > 1 {
        let board_pin_number = pin_description.board_pin_number;
        let bcm_pin_number = pin_description.bcm_pin_number.unwrap();
        let mut pin_options_row = Row::new()
            .align_items(Alignment::Center)
            .width(Length::Fixed(140f32));
        let mut config_options = pin_description.options.to_vec();
        let selected = match selected_function {
            PinFunction::None => None,
            other => {
                config_options.push(PinFunction::None);
                Some(other)
            }
        };
        pin_options_row = pin_options_row.push(
            pick_list(config_options, selected, move |pin_function| {
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
    pin_name_row = pin_name_row.push(Text::new(pin_description.name));
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
    let pin_button = button(
        Text::new(pin_description.board_pin_number.to_string())
            .size(20)
            .horizontal_alignment(Horizontal::Center),
    )
    .padding(10)
    .width(Length::Fixed(40f32))
    .style(get_pin_color(pin_description).get_button_style())
    .on_press(Message::Activate(pin_description.board_pin_number));

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
