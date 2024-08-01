use iced::advanced::text::editor::Direction;
use iced::advanced::text::editor::Direction::{Left, Right};
use iced::alignment::Horizontal;
use iced::futures::channel::mpsc::Sender;
use iced::widget::tooltip::Position;
use iced::widget::Tooltip;
use iced::widget::{button, horizontal_space, pick_list, toggler, Column, Row, Text};
use iced::{Alignment, Color, Command, Element, Length};
use iced_futures::Subscription;
use iroh_net::relay::RelayUrl;
use iroh_net::NodeId;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::time::Duration;

use crate::hardware_subscription;
use crate::hw::config::HardwareConfig;
use crate::hw::pin_description::{PinDescription, PinDescriptionSet};
use crate::hw::pin_function::PinFunction;
use crate::hw::pin_function::PinFunction::{Input, Output};
use crate::hw::HardwareConfigMessage;
use crate::hw::{BCMPinNumber, BoardPinNumber, LevelChange, PinLevel};
use crate::hw::{HardwareDescription, InputPull};
use crate::network_subscription;
use crate::styles::button_style::ButtonStyle;
use crate::styles::toggler_style::TogglerStyle;
use crate::views::hardware_view::HardwareTarget::{Local, NoHW, Remote};
use crate::views::hardware_view::HardwareViewMessage::{
    Activate, ChangeOutputLevel, HardwareSubscription, NewConfig, PinFunctionSelected, UpdateCharts,
};
use crate::views::layout_selector::Layout;
use crate::views::pin_state::{CHART_UPDATES_PER_SECOND, CHART_WIDTH};
use crate::widgets::clicker::clicker;
use crate::widgets::led::led;
use crate::widgets::{circle::circle, line::line};
use crate::{Message, Piggui, PinState};

// WIDTHS
const PIN_BUTTON_WIDTH: f32 = 30.0;
const PIN_ARROW_LINE_WIDTH: f32 = 20.0;
const PIN_ARROW_CIRCLE_RADIUS: f32 = 5.0;
const PIN_ARROW_WIDTH: f32 = PIN_ARROW_LINE_WIDTH + PIN_ARROW_CIRCLE_RADIUS * 2.0;
const PIN_NAME_WIDTH: f32 = 60.0;
const PIN_OPTION_WIDTH: f32 = 130.0;
const TOGGLER_SIZE: f32 = 30.0;
const TOGGLER_WIDTH: f32 = 95.0; // Just used to calculate Pullup width
const BUTTON_WIDTH: f32 = 16.0;
// We want the pullup on an Input to be the same width as the clicker + toggler on an Output
const PULLUP_WIDTH: f32 = TOGGLER_WIDTH + WIDGET_ROW_SPACING + BUTTON_WIDTH;
const LED_WIDTH: f32 = 16.0;
const WIDGET_ROW_SPACING: f32 = 5.0;
const PIN_WIDGET_ROW_WIDTH: f32 =
    PULLUP_WIDTH + WIDGET_ROW_SPACING + LED_WIDTH + WIDGET_ROW_SPACING + CHART_WIDTH;

// const PIN_VIEW_SIDE_WIDTH: f32 = PIN_BUTTON_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_ARROW_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_NAME_WIDTH
//     + WIDGET_ROW_SPACING
//     + PIN_OPTION_WIDTH;

const BOARD_LAYOUT_WIDTH_BETWEEN_PIN_ROWS: f32 = 10.0;
// Export these two, so they can be used to calculate overall window size
// pub const BCM_PIN_LAYOUT_WIDTH: f32 = PIN_VIEW_SIDE_WIDTH; // One pin row per row

// Board Layout has two pin rows per row, with spacing between them
// pub const BOARD_PIN_LAYOUT_WIDTH: f32 =
//     PIN_VIEW_SIDE_WIDTH + PIN_VIEW_SIDE_WIDTH + BOARD_LAYOUT_WIDTH_BETWEEN_PIN_ROWS;

// HEIGHTS
const VERTICAL_SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;
const BCM_SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;

/// This enum is for events created by async events in the hardware that will be sent to the Gui
// TODO pass PinDescriptions as a reference and handle lifetimes - clone on reception
#[allow(clippy::large_enum_variant)] // remove when fix todo above
#[derive(Clone, Debug)]
pub enum HardwareEventMessage {
    /// This event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Connected(Sender<HardwareConfigMessage>, HardwareDescription),
    /// This event indicates that the logic level of an input has just changed
    InputChange(BCMPinNumber, LevelChange),
    /// We have lost the connection to the hardware
    Disconnected(String),
}

/// [HardwareViewMessage] covers all messages that are handled by hardware_view
#[derive(Debug, Clone)]
pub enum HardwareViewMessage {
    Activate(BoardPinNumber),
    PinFunctionSelected(BCMPinNumber, PinFunction),
    NewConfig(HardwareConfig),
    HardwareSubscription(HardwareEventMessage),
    ChangeOutputLevel(BCMPinNumber, LevelChange),
    UpdateCharts,
}

fn get_pin_style(pin_description: &PinDescription) -> ButtonStyle {
    match pin_description.name.as_ref() {
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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum HardwareTarget {
    NoHW,
    #[default]
    Local,
    Remote(NodeId, Option<RelayUrl>),
}

pub struct HardwareView {
    hardware_config: HardwareConfig,
    hardware_sender: Option<Sender<HardwareConfigMessage>>,
    hardware_description: Option<HardwareDescription>,
    /// Either desired state of an output, or detected state of input.
    /// Note: Indexed by BoardPinNumber -1 (since BoardPinNumbers start at 1)
    pin_states: HashMap<BCMPinNumber, PinState>,
}

async fn empty() {}

impl HardwareView {
    pub fn new() -> Self {
        Self {
            hardware_config: HardwareConfig::default(),
            hardware_description: None, // Until listener is ready
            hardware_sender: None,      // Until listener is ready
            pin_states: HashMap::new(),
        }
    }

    pub fn get_config(&self) -> HardwareConfig {
        self.hardware_config.clone()
    }

    /// Return a String describing the HW Piggui is connected to, or a placeholder string
    #[must_use]
    pub fn hw_description(&self) -> String {
        if let Some(hardware_description) = &self.hardware_description {
            format!(
                "Hardware: {}\nRevision: {}\nSerial: {}\nModel: {}",
                hardware_description.details.hardware,
                hardware_description.details.revision,
                hardware_description.details.serial,
                hardware_description.details.model,
            )
        } else {
            "No Hardware connected".to_string()
        }
    }

    /// Return a String describing the Model of HW Piggui is connected to, or a placeholder string
    #[must_use]
    pub fn hw_model(&self) -> Option<String> {
        self.hardware_description
            .as_ref()
            .map(|desc| desc.details.model.clone())
    }

    /// Send the GPIOConfig from the GUI to the hardware to have it applied
    fn update_hw_config(&mut self) {
        if let Some(ref mut hardware_sender) = &mut self.hardware_sender {
            let _ = hardware_sender.try_send(HardwareConfigMessage::NewConfig(
                self.hardware_config.clone(),
            ));
        }
    }

    /// A new function has been selected for a pin via the UI, this function:
    /// - updates the pin_selected_function array for the UI
    /// - saves it in the gpio_config, so when we save later it's there
    /// - sends the update to the hardware to have it applied
    fn new_pin_function(&mut self, bcm_pin_number: BCMPinNumber, new_function: PinFunction) {
        let previous_function = self
            .hardware_config
            .pins
            .get(&bcm_pin_number)
            .unwrap_or(&PinFunction::None);
        if &new_function != previous_function {
            self.hardware_config
                .pins
                .insert(bcm_pin_number, new_function);

            self.pin_states.insert(bcm_pin_number, PinState::new());

            // Report config changes to the hardware listener
            // Since config loading and hardware listener setup can occur out of order
            // mark the config as changed. If we send to the listener, then mark as done
            if let Some(ref mut listener) = &mut self.hardware_sender {
                let _ = listener.try_send(HardwareConfigMessage::NewPinConfig(
                    bcm_pin_number,
                    new_function,
                ));
            }
        }
    }

    /// Go through all the pins in the [HardwareConfig], make sure a pin state exists for the pin
    /// and then set the current level if it was specified for an Output
    fn set_pin_states_after_load(&mut self) {
        for (bcm_pin_number, function) in &self.hardware_config.pins {
            // For output pins, if there is an initial state set then set that in pin state
            // so the toggler will be drawn correctly on first draw
            if let Output(Some(level)) = function {
                self.pin_states
                    .entry(*bcm_pin_number)
                    .or_insert(PinState::new())
                    .set_level(LevelChange::new(*level));
            }
        }
    }

    /// Apply a new config to the connected hardware
    pub fn new_config(&mut self, new_config: HardwareConfig) {
        self.hardware_config = new_config;
        self.set_pin_states_after_load();
        self.update_hw_config();
    }

    pub fn update(&mut self, message: HardwareViewMessage) -> Command<Message> {
        match message {
            UpdateCharts => {
                // Update all the charts of the pins that have an assigned function
                for pin in self.pin_states.values_mut() {
                    pin.chart.refresh();
                }
            }

            PinFunctionSelected(bcm_pin_number, pin_function) => {
                self.new_pin_function(bcm_pin_number, pin_function);
                return Command::perform(empty(), |_| {
                    <Piggui as iced::Application>::Message::ConfigChangesMade
                });
            }

            NewConfig(config) => {
                self.new_config(config);
            }

            HardwareSubscription(event) => match event {
                HardwareEventMessage::Connected(config_change_sender, hw_desc) => {
                    self.hardware_sender = Some(config_change_sender);
                    self.hardware_description = Some(hw_desc);
                    self.set_pin_states_after_load();
                    self.update_hw_config();
                    return Command::perform(empty(), |_| {
                        <Piggui as iced::Application>::Message::Connected
                    });
                }
                HardwareEventMessage::InputChange(bcm_pin_number, level_change) => {
                    self.pin_states
                        .entry(bcm_pin_number)
                        .or_insert(PinState::new())
                        .set_level(level_change);
                }
                HardwareEventMessage::Disconnected(message) => {
                    return Command::perform(empty(), |_| {
                        <Piggui as iced::Application>::Message::ConnectionError(message)
                    });
                }
            },

            ChangeOutputLevel(bcm_pin_number, level_change) => {
                self.pin_states
                    .entry(bcm_pin_number)
                    .or_insert(PinState::new())
                    .set_level(level_change.clone());
                if let Some(ref mut listener) = &mut self.hardware_sender {
                    let _ = listener.try_send(HardwareConfigMessage::IOLevelChanged(
                        bcm_pin_number,
                        level_change,
                    ));
                }
            }

            Activate(pin_number) => println!("Pin {pin_number} clicked"),
        }

        Command::none()
    }

    pub fn view(
        &self,
        layout: Layout,
        hardware_target: &HardwareTarget,
    ) -> Element<HardwareViewMessage> {
        if hardware_target == &NoHW {
            return Row::new().into();
        }

        if let Some(hw_description) = &self.hardware_description {
            let pin_layout = match layout {
                Layout::BoardLayout => self.board_pin_layout_view(&hw_description.pins),
                Layout::BCMLayout => self.bcm_pin_layout_view(&hw_description.pins),
            };

            return pin_layout;
        }

        // The no hardware view will go here and maybe some widget to search for and connect to remote HW?
        Row::new().into()
    }

    /// Create subscriptions for ticks for updating charts of waveforms and events coming from hardware
    pub fn subscription(
        &self,
        hardware_target: &HardwareTarget,
    ) -> Subscription<HardwareViewMessage> {
        let mut subscriptions =
            vec![
                iced::time::every(Duration::from_millis(1000 / CHART_UPDATES_PER_SECOND))
                    .map(|_| UpdateCharts),
            ];

        match hardware_target {
            NoHW => {}
            Local => {
                subscriptions.push(hardware_subscription::subscribe().map(HardwareSubscription));
            }
            Remote(nodeid, relay) => {
                subscriptions.push(
                    network_subscription::subscribe(*nodeid, relay.clone())
                        .map(HardwareSubscription),
                );
            }
        }

        Subscription::batch(subscriptions)
    }

    /// View that lays out the pins in a single column ordered by BCM pin number
    pub fn bcm_pin_layout_view<'a>(
        &'a self,
        pin_set: &'a PinDescriptionSet,
    ) -> Element<'a, HardwareViewMessage> {
        let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

        for pin_description in pin_set.bcm_pins_sorted() {
            let pin_row = create_pin_view_side(
                pin_description,
                self.hardware_config.pins.get(&pin_description.bcm.unwrap()),
                Right,
                self.pin_states.get(&pin_description.bcm.unwrap_or(0)),
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
        &'a self,
        pin_descriptions: &'a PinDescriptionSet,
    ) -> Element<'a, HardwareViewMessage> {
        let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

        // Draw all pins, those with and without [BCMPinNumber]
        for pair in pin_descriptions.pins().chunks(2) {
            let left_row = create_pin_view_side(
                &pair[0],
                self.hardware_config.pins.get(&pair[0].bcm.unwrap_or(0)),
                Left,
                self.pin_states.get(&pair[0].bcm.unwrap_or(0)),
            );

            let right_row = create_pin_view_side(
                &pair[1],
                self.hardware_config.pins.get(&pair[1].bcm.unwrap_or(0)),
                Right,
                self.pin_states.get(&pair[1].bcm.unwrap_or(0)),
            );

            let row = Row::new()
                .push(left_row)
                .push(right_row)
                .spacing(BOARD_LAYOUT_WIDTH_BETWEEN_PIN_ROWS)
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
}

/// Prepare a pick_list widget with the Input's pullup options
fn pullup_picklist(
    pull: &Option<InputPull>,
    bcm_pin_number: BCMPinNumber,
) -> Element<'static, HardwareViewMessage> {
    let mut sub_options = vec![InputPull::PullUp, InputPull::PullDown, InputPull::None];

    // Filter out the currently selected pull option
    if let Some(selected_pull) = pull {
        sub_options.retain(|&option| option != *selected_pull);
    }

    pick_list(sub_options, *pull, move |selected_pull| {
        PinFunctionSelected(bcm_pin_number, Input(Some(selected_pull)))
    })
    .width(Length::Fixed(PULLUP_WIDTH))
    .placeholder("Select Pullup")
    .into()
}

/// Create the widget that either shows an input pin's state,
/// or allows the user to control the state of an output pin
/// This should only be called for pins that have a valid BCMPinNumber
fn get_pin_widget<'a>(
    bcm_pin_number: Option<BCMPinNumber>,
    pin_function: Option<&'a PinFunction>,
    pin_state: &'a PinState,
    direction: Direction,
) -> Element<'a, HardwareViewMessage> {
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

    let row: Row<HardwareViewMessage> = match pin_function {
        Some(Input(pull)) => {
            let pullup_pick = pullup_picklist(pull, bcm_pin_number.unwrap());
            if direction == Left {
                Row::new()
                    .push(pin_state.view(Left))
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(pullup_pick)
            } else {
                Row::new()
                    .push(pullup_pick)
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(pin_state.view(Right))
            }
        }

        Some(Output(_)) => {
            let output_toggler = toggler(
                None,
                pin_state.get_level().unwrap_or(false as PinLevel),
                move |b| ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(b)),
            )
            .size(TOGGLER_SIZE)
            .style(toggle_button_style.get_toggler_style());

            let output_clicker =
                clicker::<HardwareViewMessage>(BUTTON_WIDTH, Color::BLACK, Color::WHITE)
                    .on_press({
                        let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                        ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level))
                    })
                    .on_release({
                        let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                        ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level))
                    });

            let toggle_tooltip =
                Tooltip::new(output_toggler, "Click to toggle level", Position::Top);

            let clicker_tooltip = Tooltip::new(
                output_clicker,
                "Click and hold to invert level",
                Position::Top,
            );

            // For some unknown reason the Pullup picker is wider on the right side than the left
            // to we add some space here to make this match on both side. A nasty hack!
            if direction == Left {
                Row::new()
                    .push(pin_state.view(Left))
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(clicker_tooltip)
                    .push(toggle_tooltip)
            } else {
                Row::new()
                    .push(toggle_tooltip)
                    .push(clicker_tooltip)
                    .push(horizontal_space().width(Length::Fixed(4.0))) // HACK!
                    .push(led(LED_WIDTH, LED_WIDTH, pin_state.get_level()))
                    .push(pin_state.view(Right))
            }
        }

        _ => Row::new(),
    };

    row.width(Length::Fixed(PIN_WIDGET_ROW_WIDTH))
        .spacing(WIDGET_ROW_SPACING)
        .align_items(Alignment::Center)
        .into()
}

// Filter options for PickList
fn filter_options(
    options: &[PinFunction],
    selected_function: Option<PinFunction>,
) -> Vec<PinFunction> {
    let mut config_options: Vec<_> = options
        .iter()
        .filter(|&&option| match selected_function {
            Some(PinFunction::Input(Some(_))) => {
                matches!(option, PinFunction::Output(None) | PinFunction::None)
            }
            Some(PinFunction::Output(Some(_))) => {
                matches!(option, PinFunction::Input(None) | PinFunction::None)
            }
            Some(selected) => selected != option,
            None => option != PinFunction::None,
        })
        .cloned()
        .collect();

    if !config_options.contains(&PinFunction::None)
        && selected_function.is_some()
        && selected_function != Some(PinFunction::None)
    {
        config_options.push(PinFunction::None);
    }

    config_options
}

/// Create a row of widgets that represent a pin, either from left to right or right to left
fn create_pin_view_side<'a>(
    pin_description: &'a PinDescription,
    selected_function: Option<&'a PinFunction>,
    direction: Direction,
    pin_state: Option<&'a PinState>,
) -> Row<'a, HardwareViewMessage> {
    let pin_widget = if let Some(state) = pin_state {
        // Create a widget that is either used to visualize an input or control an output
        get_pin_widget(pin_description.bcm, selected_function, state, direction)
    } else {
        Row::new().width(Length::Fixed(PIN_WIDGET_ROW_WIDTH)).into()
    };

    // Create the drop-down selector of pin function
    let mut pin_option = Column::new()
        .width(Length::Fixed(PIN_OPTION_WIDTH))
        .align_items(Alignment::Center);

    if pin_description.options.len() > 1 {
        let bcm_pin_number = pin_description.bcm.unwrap();
        let mut pin_options_row = Row::new().align_items(Alignment::Center);

        // Filter options
        let config_options = filter_options(&pin_description.options, selected_function.cloned());

        let selected = selected_function.filter(|&pin_function| *pin_function != PinFunction::None);

        pin_options_row = pin_options_row.push(
            pick_list(config_options, selected, move |pin_function| {
                PinFunctionSelected(bcm_pin_number, pin_function)
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
        .push(Text::new(pin_description.name.to_string()))
        .align_items(Alignment::Center);

    pin_name_column = pin_name_column.push(pin_name).width(PIN_NAME_WIDTH);

    let mut pin_arrow = Row::new()
        .align_items(Alignment::Center)
        .width(Length::Fixed(PIN_ARROW_WIDTH));

    if direction == Left {
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
    } else {
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
    }

    let mut pin_button_column = Column::new().align_items(Alignment::Center);
    // Create the pin itself, with number and as a button
    let pin_button =
        button(Text::new(pin_description.bpn.to_string()).horizontal_alignment(Horizontal::Center))
            .width(Length::Fixed(PIN_BUTTON_WIDTH))
            .style(get_pin_style(pin_description).get_button_style())
            .on_press(Activate(pin_description.bpn));

    pin_button_column = pin_button_column.push(pin_button);
    // Create the row of widgets that represent the pin, inverted order if left or right
    let row = if direction == Left {
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

#[cfg(test)]
mod test {
    use crate::views::hardware_view::HardwareView;

    #[test]
    fn no_hardware_description() {
        let hw_view = HardwareView::new();
        assert_eq!(hw_view.hw_description(), "No Hardware connected");
    }

    #[test]
    fn no_hardware_model() {
        let hw_view = HardwareView::new();
        assert_eq!(hw_view.hw_model(), None);
    }

    #[test]
    fn test_filter_options() {
        use super::*;

        let options = vec![
            PinFunction::Input(None),
            PinFunction::Output(None),
            PinFunction::None,
        ];

        // Test case: No function selected
        let result = filter_options(&options, None);
        assert_eq!(
            result,
            vec![PinFunction::Input(None), PinFunction::Output(None)]
        );

        // Test case: Input selected
        let result = filter_options(&options, Some(PinFunction::Input(None)));
        assert_eq!(result, vec![PinFunction::Output(None), PinFunction::None]);

        // Test case: Output selected
        let result = filter_options(&options, Some(PinFunction::Output(None)));
        assert_eq!(result, vec![PinFunction::Input(None), PinFunction::None]);

        // Test case: None selected
        let result = filter_options(&options, Some(PinFunction::None));
        assert_eq!(
            result,
            vec![PinFunction::Input(None), PinFunction::Output(None)]
        );
    }
}
