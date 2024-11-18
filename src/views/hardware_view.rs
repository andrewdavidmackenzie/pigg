use crate::hardware_subscription;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::InputPull;
use crate::hw_definition::description::{HardwareDescription, PinDescription, PinDescriptionSet};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::{Input, Output};
use crate::hw_definition::{config::LevelChange, BCMPinNumber, BoardPinNumber, PinLevel};
use crate::views::hardware_styles::{
    get_pin_style, CLICKER_WIDTH, LED_WIDTH, PIN_ARROW_CIRCLE_RADIUS, PIN_ARROW_LINE_WIDTH,
    PIN_ARROW_WIDTH, PIN_BUTTON_WIDTH, PIN_NAME_WIDTH, PIN_OPTION_WIDTH, PIN_WIDGET_ROW_WIDTH,
    PULLUP_WIDTH, SPACE_BETWEEN_PIN_COLUMNS, SPACE_BETWEEN_PIN_ROWS, TOGGLER_HOVER_STYLE,
    TOGGLER_SIZE, TOGGLER_STYLE, TOGGLER_WIDTH, TOOLTIP_STYLE, WIDGET_ROW_SPACING,
};
use crate::views::hardware_view::HardwareTarget::*;
use crate::views::hardware_view::HardwareViewMessage::{
    Activate, ChangeOutputLevel, HardwareSubscription, NewConfig, PinFunctionSelected, UpdateCharts,
};
use crate::views::layout_menu::Layout;
use crate::views::pin_state::{PinState, CHART_UPDATES_PER_SECOND};
use crate::widgets::clicker::clicker;
use crate::widgets::led::led;
use crate::widgets::{circle::circle, line::line};
use crate::Message;
use iced::advanced::text::editor::Direction;
use iced::advanced::text::editor::Direction::{Left, Right};
use iced::futures::channel::mpsc::Sender;
use iced::widget::scrollable::Scrollbar;
use iced::widget::toggler::Status::Hovered;
use iced::widget::tooltip::Position;
use iced::widget::{button, horizontal_space, pick_list, scrollable, toggler, Column, Row, Text};
use iced::widget::{container, Tooltip};
use iced::Color;
use iced::{Alignment, Center, Element, Length, Task};
use iced_futures::Subscription;
#[cfg(feature = "iroh")]
use iroh_net::{relay::RelayUrl, NodeId};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// This enum is for async events in the hardware that will be sent to the GUI
#[allow(clippy::large_enum_variant)]
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

#[derive(Debug, Clone, Default, PartialEq)]
pub enum HardwareTarget {
    #[cfg_attr(target_arch = "wasm32", default)]
    NoHW,
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(not(target_arch = "wasm32"), default)]
    Local,
    #[cfg(feature = "iroh")]
    Iroh(NodeId, Option<RelayUrl>),
    #[cfg(feature = "tcp")]
    Tcp(core::net::IpAddr, u16),
}

pub struct HardwareView {
    hardware_config: HardwareConfig,
    hardware_sender: Option<Sender<HardwareConfigMessage>>,
    pub hardware_description: Option<HardwareDescription>,
    /// Either desired state of an output, or detected state of input.
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
            .pin_functions
            .get(&bcm_pin_number)
            .unwrap_or(&PinFunction::None);
        if &new_function != previous_function {
            self.hardware_config
                .pin_functions
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
    /// and then set the current level if pin is an Output and the level was specified.
    /// TODO check if still needed - or should we add level reading to apply_config_change()
    /// TODO at the end of hardware_subscription.rs and keep this out of the UI code.
    /// TODO that should take care of states for outputs also. If we move that in there, then
    /// TODO it should use hardware::get_time_since_boot()
    fn set_pin_states_after_load(&mut self) {
        for (bcm_pin_number, pin_function) in &self.hardware_config.pin_functions {
            // For output pins, if there is an initial state set then set that in pin state
            // so the toggler will be drawn correctly on first draw
            if let Output(Some(level)) = pin_function {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                self.pin_states
                    .entry(*bcm_pin_number)
                    .or_insert(PinState::new())
                    .set_level(LevelChange::new(*level, now));
            }
        }
    }

    /// Apply a new config to the connected hardware
    pub fn new_config(&mut self, new_config: HardwareConfig) {
        self.hardware_config = new_config;
        self.set_pin_states_after_load();
        self.update_hw_config();
    }

    pub fn update(&mut self, message: HardwareViewMessage) -> Task<Message> {
        match message {
            UpdateCharts => {
                // Update all the charts of the pins that have an assigned function
                for pin in self.pin_states.values_mut() {
                    pin.chart.refresh();
                }
            }

            PinFunctionSelected(bcm_pin_number, pin_function) => {
                self.new_pin_function(bcm_pin_number, pin_function);
                return Task::perform(empty(), |_| Message::ConfigChangesMade);
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
                    return Task::perform(empty(), |_| Message::Connected);
                }
                HardwareEventMessage::InputChange(bcm_pin_number, level_change) => {
                    self.pin_states
                        .entry(bcm_pin_number)
                        .or_insert(PinState::new())
                        .set_level(level_change);
                }
                HardwareEventMessage::Disconnected(details) => {
                    return Task::perform(empty(), move |_| {
                        Message::ConnectionError(details.clone())
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

        Task::none()
    }

    fn hw_view(
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

            return scrollable(pin_layout)
                .direction({
                    let scrollbar = Scrollbar::new().width(10);
                    scrollable::Direction::Both {
                        horizontal: scrollbar,
                        vertical: scrollbar,
                    }
                })
                .into();
        }

        // The no hardware view will go here and maybe some widget to search for and connect to remote HW?
        Row::new().into()
    }

    /// Construct the view that represents the main row of the app
    pub fn view<'a>(
        &'a self,
        layout: Layout,
        hardware_target: &'a HardwareTarget,
    ) -> Element<'a, Message> {
        let hw_column = Column::new()
            .push(self.hw_view(layout, hardware_target).map(Message::Hardware))
            .align_x(Center)
            .height(Length::Fill)
            .width(Length::Fill);

        container(hw_column).padding(10.0).into()
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

        if hardware_target != &NoHW {
            subscriptions.push(
                Subscription::run_with_id(
                    "hardware",
                    hardware_subscription::subscribe(hardware_target),
                )
                .map(HardwareSubscription),
            );
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
            if let Some(bcm_pin_number) = &pin_description.bcm {
                let pin_row = create_pin_view_side(
                    pin_description,
                    self.hardware_config
                        .pin_functions
                        .get(&pin_description.bcm.unwrap()),
                    Right,
                    self.pin_states.get(bcm_pin_number),
                );

                column = column.push(pin_row);
            }
        }

        column
            .spacing(SPACE_BETWEEN_PIN_ROWS)
            .align_x(Alignment::Start)
            .into()
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
                pair[0]
                    .bcm
                    .and_then(|bcm| self.hardware_config.pin_functions.get(&bcm)),
                Left,
                pair[0].bcm.and_then(|bcm| self.pin_states.get(&bcm)),
            );

            let right_row = create_pin_view_side(
                &pair[1],
                pair[1]
                    .bcm
                    .and_then(|bcm| self.hardware_config.pin_functions.get(&bcm)),
                Right,
                pair[1].bcm.and_then(|bcm| self.pin_states.get(&bcm)),
            );

            let row = Row::new()
                .push(left_row)
                .push(right_row)
                .spacing(SPACE_BETWEEN_PIN_COLUMNS)
                .align_y(Center);

            column = column.push(row);
        }

        column
            .spacing(SPACE_BETWEEN_PIN_ROWS)
            .align_x(Center)
            .into()
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

    let pick_list = pick_list(sub_options, *pull, move |selected_pull| {
        PinFunctionSelected(bcm_pin_number, Input(Some(selected_pull)))
    })
    .width(Length::Fixed(PULLUP_WIDTH))
    .placeholder("Select Pullup");

    // select a slightly small font on RPi, to make it fit within pick_list

    pick_list.into()
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
            let output_toggler = toggler(pin_state.get_level().unwrap_or(false as PinLevel))
                .on_toggle(move |b| {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(b, now))
                })
                .width(TOGGLER_WIDTH)
                .size(TOGGLER_SIZE)
                .style(move |_theme, status| match status {
                    Hovered { .. } => TOGGLER_HOVER_STYLE,
                    _ => TOGGLER_STYLE,
                });

            let output_clicker =
                clicker::<HardwareViewMessage>(CLICKER_WIDTH, Color::BLACK, Color::WHITE)
                    .on_press({
                        let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level, now))
                    })
                    .on_release({
                        let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        ChangeOutputLevel(bcm_pin_number.unwrap(), LevelChange::new(!level, now))
                    });

            let toggle_tooltip =
                Tooltip::new(output_toggler, "Click to toggle level", Position::Top)
                    .gap(4.0)
                    .style(|_| TOOLTIP_STYLE);

            let clicker_tooltip = Tooltip::new(
                output_clicker,
                "Click and hold to invert level",
                Position::Top,
            )
            .gap(4.0)
            .style(|_| TOOLTIP_STYLE);

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
        .align_y(Center)
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
            Some(Input(Some(_))) => {
                matches!(option, Output(None) | PinFunction::None)
            }
            Some(Output(Some(_))) => {
                matches!(option, Input(None) | PinFunction::None)
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
    pin_function: Option<&'a PinFunction>,
    direction: Direction,
    pin_state: Option<&'a PinState>,
) -> Row<'a, HardwareViewMessage> {
    let pin_widget = if let Some(state) = pin_state {
        // Create a widget that is either used to visualize an input or control an output
        get_pin_widget(pin_description.bcm, pin_function, state, direction)
    } else {
        Row::new().width(Length::Fixed(PIN_WIDGET_ROW_WIDTH)).into()
    };

    // Create the drop-down selector of pin function
    let mut pin_option = Column::new()
        .width(Length::Fixed(PIN_OPTION_WIDTH))
        .align_x(Center);

    if let Some(bcm_pin_number) = pin_description.bcm {
        let mut pin_options_row = Row::new().align_y(Center);

        // Filter options
        let config_options = filter_options(&pin_description.options, pin_function.cloned());

        if !config_options.is_empty() {
            let selected = pin_function.filter(|&pin_function| *pin_function != PinFunction::None);

            let pick_list = pick_list(config_options, selected, move |pin_function| {
                PinFunctionSelected(bcm_pin_number, pin_function)
            })
            .width(Length::Fixed(PIN_OPTION_WIDTH))
            .placeholder("Select function");

            // select a slightly small font on RPi, to make it fit within pick_list
            pin_options_row = pin_options_row.push(pick_list);
        }

        pin_option = pin_option.push(pin_options_row);
    }

    let mut pin_name_column = Column::new()
        .width(Length::Fixed(PIN_NAME_WIDTH))
        .align_x(Center);

    // Create the Pin name
    let pin_name = Row::new()
        .push(Text::new(pin_description.name.to_string()))
        .align_y(Center);

    pin_name_column = pin_name_column.push(pin_name).width(PIN_NAME_WIDTH);

    let mut pin_arrow = Row::new()
        .align_y(Center)
        .width(Length::Fixed(PIN_ARROW_WIDTH));

    if direction == Left {
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
    } else {
        pin_arrow = pin_arrow.push(line(PIN_ARROW_LINE_WIDTH));
        pin_arrow = pin_arrow.push(circle(PIN_ARROW_CIRCLE_RADIUS));
    }

    let mut pin_button_column = Column::new().align_x(Center);
    // Create the pin itself, with number and as a button
    let pin_button = button(
        container(Text::new(pin_description.bpn.to_string()))
            .align_x(Center)
            .align_y(Center),
    )
    .padding(0.0)
    .width(Length::Fixed(PIN_BUTTON_WIDTH))
    .height(Length::Fixed(PIN_BUTTON_WIDTH))
    .style(move |_, _| get_pin_style(pin_description))
    .on_press(Activate(pin_description.bpn));

    pin_button_column = pin_button_column.push(pin_button);
    // Create the row of widgets that represent the pin, inverted order if left or right
    let row = if direction == Left {
        Row::new()
            .push(pin_widget)
            .push(pin_option)
            .push(pin_name_column.align_x(Alignment::End))
            .push(pin_arrow)
            .push(pin_button_column)
    } else {
        Row::new()
            .push(pin_button_column)
            .push(pin_arrow)
            .push(pin_name_column.align_x(Alignment::Start))
            .push(pin_option)
            .push(pin_widget)
    };

    row.align_y(Center).spacing(WIDGET_ROW_SPACING)
}

#[cfg(test)]
mod test {
    use crate::views::hardware_view::HardwareView;

    #[test]
    fn no_hardware_description() {
        let hw_view = HardwareView::new();
        assert!(hw_view.hardware_description.is_none());
    }

    #[test]
    fn no_hardware_model() {
        let hw_view = HardwareView::new();
        assert_eq!(hw_view.hw_model(), None);
    }

    #[test]
    fn test_filter_options() {
        use super::*;

        let options = vec![Input(None), Output(None), PinFunction::None];

        // Test case: No function selected
        let result = filter_options(&options, None);
        assert_eq!(result, vec![Input(None), Output(None)]);

        // Test case: Input selected
        let result = filter_options(&options, Some(Input(None)));
        assert_eq!(result, vec![Output(None), PinFunction::None]);

        // Test case: Output selected
        let result = filter_options(&options, Some(Output(None)));
        assert_eq!(result, vec![Input(None), PinFunction::None]);

        // Test case: None selected
        let result = filter_options(&options, Some(PinFunction::None));
        assert_eq!(result, vec![Input(None), Output(None)]);
    }
}
