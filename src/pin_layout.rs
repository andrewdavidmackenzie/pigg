use iced::{Alignment, Color, Element, Length};
use iced::advanced::text::editor::Direction;
use iced::alignment::Horizontal;
use iced::widget::{button, Column, pick_list, Row, Text, toggler};
use iced::widget::mouse_area;

use crate::{Gpio, PinState};
use crate::custom_widgets::{circle::circle, line::line};
use crate::custom_widgets::button_style::ButtonStyle;
use crate::custom_widgets::clicker::clicker;
use crate::custom_widgets::led::led;
use crate::custom_widgets::toggler_style::TogglerStyle;
use crate::hw::{
    BCMPinNumber, BoardPinNumber, LevelChange, PinDescription, PinDescriptionSet, PinFunction,
    PinLevel,
};
use crate::hw::InputPull;
use crate::hw::PinFunction::{Input, Output};
use crate::Message;
use crate::pin_state::CHART_WIDTH;

const PIN_ARROW_LINE_WIDTH: f32 = 20.0;
const PIN_OPTION_WIDTH: f32 = 130.0;
const PIN_ARROW_CIRCLE_RADIUS: f32 = 5.0;
const VERTICAL_SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;
const BOARD_SPACE_BETWEEN_PIN_ROWS: f32 = 10.0;
const BCM_SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;
const PIN_NAME_WIDTH: f32 = 60.0;
const PIN_ARROW_WIDTH: f32 = PIN_ARROW_LINE_WIDTH + PIN_ARROW_CIRCLE_RADIUS * 2.0;
const PIN_BUTTON_WIDTH: f32 = 30.0;
const LED_WIDTH: f32 = 16.0;
const BUTTON_WIDTH: f32 = 16.0;
const TOGGLER_WIDTH: f32 = 100.0;
const PULLUP_WIDTH: f32 = TOGGLER_WIDTH + BUTTON_WIDTH;
const WIDGET_ROW_SPACING: f32 = 5.0;
const PIN_WIDGET_ROW_WIDTH: f32 =
    PULLUP_WIDTH + WIDGET_ROW_SPACING + LED_WIDTH + WIDGET_ROW_SPACING + CHART_WIDTH;
const PIN_VIEW_SIDE_WIDTH: f32 = PIN_BUTTON_WIDTH
    + WIDGET_ROW_SPACING
    + PIN_ARROW_WIDTH
    + WIDGET_ROW_SPACING
    + PIN_NAME_WIDTH
    + WIDGET_ROW_SPACING
    + PIN_OPTION_WIDTH;
const BCM_LAYOUT_WIDTH: f32 = PIN_VIEW_SIDE_WIDTH; // One pin row per row

// Board Layout has two pin rows per row, with spacing between them
const BOARD_LAYOUT_WIDTH: f32 =
    PIN_VIEW_SIDE_WIDTH + PIN_VIEW_SIDE_WIDTH + BOARD_SPACE_BETWEEN_PIN_ROWS;

fn get_pin_style(pin_description: &PinDescription) -> ButtonStyle {
    match pin_description.name {
        "3V3" => ButtonStyle {
            bg_color: Color::new(1.0, 0.92, 0.016, 1.0), // Yellow
            text_color: Color::BLACK,
            border_radius: 50.0,
            hovered_bg_color: Color::new(1.0, 1.0, 0.0, 1.0),
            hovered_text_color: Color::BLACK,
        },
        "5V" => ButtonStyle {
            bg_color: Color::new(1.0, 0.0, 0.0, 1.0), // Red
            text_color: Color::BLACK,
            border_radius: 50.0,
            hovered_bg_color: Color::new(1.0, 0.0, 0.0, 1.0),
            hovered_text_color: Color::BLACK,
        },
        "Ground" => ButtonStyle {
            bg_color: Color::BLACK,
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::BLACK,
        },

        "GPIO2" | "GPIO3" => ButtonStyle {
            bg_color: Color::new(0.678, 0.847, 0.902, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.678, 0.847, 0.902, 1.0),
        },

        "GPIO7" | "GPIO8" | "GPIO9" | "GPIO10" | "GPIO11" => ButtonStyle {
            bg_color: Color::new(0.933, 0.510, 0.933, 1.0), // Violet
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.933, 0.510, 0.933, 1.0),
        },

        "GPIO14" | "GPIO15" => ButtonStyle {
            bg_color: Color::new(0.0, 0.502, 0.0, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.0, 0.502, 0.0, 1.0),
        },

        "ID_SD" | "ID_SC" => ButtonStyle {
            bg_color: Color::new(0.502, 0.502, 0.502, 1.0), // Grey
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(0.502, 0.502, 0.502, 1.0),
        },
        _ => ButtonStyle {
            bg_color: Color::new(1.0, 0.647, 0.0, 1.0),
            text_color: Color::WHITE,
            border_radius: 50.0,
            hovered_bg_color: Color::WHITE,
            hovered_text_color: Color::new(1.0, 0.647, 0.0, 1.0),
        },
    }
}

/// View that lays out the pins in a single column ordered by BCM pin number
pub fn bcm_pin_layout_view<'a>(
    pin_set: &'a PinDescriptionSet,
    gpio: &'a Gpio,
) -> Element<'a, Message> {
    let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

    for pin in pin_set.bcm_pins_sorted() {
        let pin_row = create_pin_view_side(
            pin,
            gpio.pin_function_selected[pin.board_pin_number as usize - 1],
            false,
            &gpio.pin_states[pin.board_pin_number as usize - 1],
        );

        column = column
            .push(pin_row)
            .spacing(BCM_SPACE_BETWEEN_PIN_ROWS)
            .align_items(Alignment::Center);
    }

    column.into()
}

/// View that draws the pins laid out as they are on the physical Pi board
pub fn board_pin_layout_view<'a>(
    pin_descriptions: &'a PinDescriptionSet,
    gpio: &'a Gpio,
) -> Element<'a, Message> {
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
            .spacing(BOARD_SPACE_BETWEEN_PIN_ROWS)
            .align_items(Alignment::Center);

        column = column
            .push(row)
            .push(iced::widget::Space::new(
                Length::Fixed(1.0),
                Length::Fixed(VERTICAL_SPACE_BETWEEN_PIN_ROWS),
            ))
            .align_items(Alignment::Center);
    }

    column.into()
}

/// Prepare a pick_list widget with the Input's pullup options
fn pullup_picklist(
    pull: Option<InputPull>,
    board_pin_number: BoardPinNumber,
    bcm_pin_number: BCMPinNumber,
) -> Element<'static, Message> {
    let mut sub_options = vec![InputPull::PullUp, InputPull::PullDown, InputPull::None];

    // Filter out the currently selected pull option
    if let Some(selected_pull) = pull {
        sub_options.retain(|&option| option != selected_pull);
    }

    pick_list(sub_options, pull, move |selected_pull| {
        Message::PinFunctionSelected(board_pin_number, bcm_pin_number, Input(Some(selected_pull)))
    })
    .width(Length::Fixed(PULLUP_WIDTH))
    .placeholder("Select Pullup")
    .into()
}

/// Create the widget that either shows an input pin's state,
/// or allows the user to control the state of an output pin
/// This should only be called for pins that have a valid BCMPinNumber
fn get_pin_widget(
    board_pin_number: BoardPinNumber,
    bcm_pin_number: Option<BCMPinNumber>,
    pin_function: PinFunction,
    pin_state: &PinState,
    is_left: bool,
) -> Element<Message> {
    let toggle_button_style = TogglerStyle {
        background: Color::new(0.0, 0.3, 0.0, 1.0), // Dark green background (inactive)
        background_border_width: 1.0,
        background_border_color: Color::new(0.0, 0.2, 0.0, 1.0), // Darker green border (inactive)
        foreground: Color::new(1.0, 0.9, 0.8, 1.0), // Light yellowish foreground (inactive)
        foreground_border_width: 1.0,
        foreground_border_color: Color::new(0.9, 0.9, 0.9, 1.0), // Light gray foreground border (inactive)
        active_background: Color::new(0.0, 0.7, 0.0, 1.0), // Vibrant green background (active)
        active_foreground: Color::new(0.0, 0.0, 0.0, 1.0), // Black foreground (active)
        active_background_border: Color::new(0.0, 0.5, 0.0, 1.0), // Darker green border (active)
        active_foreground_border: Color::new(0.9, 0.9, 0.9, 1.0), // Light gray foreground border (active)
    };

    let row = match pin_function {
        Input(pull) => {
            let pullup_pick = pullup_picklist(pull, board_pin_number, bcm_pin_number.unwrap());
            if is_left {
                Row::new()
                    .push(pin_state.chart(Direction::Left))
                    .push(led(16.0, 16.0, pin_state.get_level()))
                    .push(pullup_pick)
            } else {
                Row::new()
                    .push(pullup_pick)
                    .push(led(16.0, 16.0, pin_state.get_level()))
                    .push(pin_state.chart(Direction::Right))
            }
        }

        Output(_) => {
            let output_control = toggler(
                None,
                pin_state.get_level().unwrap_or(false as PinLevel),
                move |b| Message::ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(b)),
            )
            .size(30)
            .style(toggle_button_style.get_toggler_style())
            .width(Length::Fixed(TOGGLER_WIDTH));

            let push_button = mouse_area(clicker(BUTTON_WIDTH))
                .on_press({
                    let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                    Message::ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level))
                })
                .on_release({
                    let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                    Message::ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level))
                });

            if is_left {
                Row::new()
                    .push(pin_state.chart(Direction::Left))
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(push_button)
                    .push(output_control)
                    .spacing(WIDGET_ROW_SPACING)
            } else {
                Row::new()
                    .push(output_control)
                    .push(push_button)
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(pin_state.chart(Direction::Right))
                    .spacing(WIDGET_ROW_SPACING)
            }
        }

        _ => Row::new(),
    };

    row.width(Length::Fixed(PIN_WIDGET_ROW_WIDTH))
        .spacing(WIDGET_ROW_SPACING)
        .align_items(Alignment::Center)
        .into()
}

/// Create a row of widgets that represent a pin, either from left to right or right to left
fn create_pin_view_side<'a>(
    pin_description: &'a PinDescription,
    selected_function: PinFunction,
    is_left: bool,
    pin_state: &'a PinState,
) -> Row<'a, Message> {
    // Create a widget that is either used to visualize an input or control an output
    let pin_widget = get_pin_widget(
        pin_description.board_pin_number,
        pin_description.bcm_pin_number,
        selected_function,
        pin_state,
        is_left,
    );

    // Create the drop-down selector of pin function
    let mut pin_option = Column::new()
        .width(Length::Fixed(PIN_OPTION_WIDTH))
        .align_items(Alignment::Center);
    if pin_description.options.len() > 1 {
        let board_pin_number = pin_description.board_pin_number;
        let bcm_pin_number = pin_description.bcm_pin_number.unwrap();
        let mut pin_options_row = Row::new().align_items(Alignment::Center);
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
            .width(Length::Fixed(PIN_OPTION_WIDTH))
            .placeholder("Select function"),
        );

        pin_option = pin_option.push(pin_options_row);
    }

    let mut pin_name_column = Column::new()
        .width(Length::Fixed(PIN_NAME_WIDTH))
        .align_items(Alignment::Center);

    // Create the Pin name
    let pin_name = Row::new()
        .push(Text::new(pin_description.name))
        .align_items(Alignment::Center);

    pin_name_column = pin_name_column.push(pin_name).width(PIN_NAME_WIDTH);

    let mut pin_arrow = Row::new()
        .align_items(Alignment::Center)
        .width(Length::Fixed(PIN_ARROW_WIDTH));

    if is_left {
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
    } else {
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
    }

    let mut pin_button_column = Column::new().align_items(Alignment::Center);
    // Create the pin itself, with number and as a button
    let pin_button = button(
        Text::new(pin_description.board_pin_number.to_string())
            .horizontal_alignment(Horizontal::Center),
    )
    .width(Length::Fixed(PIN_BUTTON_WIDTH))
    .style(get_pin_style(pin_description).get_button_style())
    .on_press(Message::Activate(pin_description.board_pin_number));

    pin_button_column = pin_button_column.push(pin_button);
    // Create the row of widgets that represent the pin, inverted order if left or right
    let row = if is_left {
        Row::new()
            .push(pin_widget)
            .push(pin_option)
            .push(pin_name_column.align_items(Alignment::End))
            .push(pin_arrow)
            .push(pin_button_column)
    } else {
        Row::new()
            .push(pin_button_column)
            .push(pin_arrow)
            .push(pin_name_column.align_items(Alignment::Start))
            .push(pin_option)
            .push(pin_widget)
    };

    row.align_items(Alignment::Center)
        .spacing(WIDGET_ROW_SPACING)
}
