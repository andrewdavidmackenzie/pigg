use crate::hardware_subscription;
use crate::hardware_subscription::SubscriberMessage::Hardware;
use crate::hardware_subscription::{SubscriberMessage, SubscriptionEvent};
use crate::hw_definition::config::InputPull;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
#[cfg(feature = "usb")]
use crate::hw_definition::description::SerialNumber;
use crate::hw_definition::description::{HardwareDescription, PinDescription, PinDescriptionSet};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::{Input, Output};
use crate::hw_definition::{config::LevelChange, BCMPinNumber, BoardPinNumber, PinLevel};
use crate::views::hardware_styles::{
    get_pin_style, CLICKER_WIDTH, LED_WIDTH, PIN_ARROW_CIRCLE_RADIUS, PIN_ARROW_LINE_WIDTH,
    PIN_ARROW_WIDTH, PIN_BUTTON_WIDTH, PIN_NAME_WIDTH, PIN_OPTION_WIDTH, PIN_WIDGET_ROW_WIDTH,
    PULLUP_WIDTH, SPACE_BETWEEN_PIN_COLUMNS, TOGGLER_HOVER_STYLE, TOGGLER_SIZE, TOGGLER_STYLE,
    TOGGLER_WIDTH, TOOLTIP_STYLE, WIDGET_ROW_SPACING,
};
use crate::views::hardware_view::HardwareConnection::*;
use crate::views::hardware_view::HardwareViewMessage::{
    Activate, ChangeOutputLevel, NewConfig, PinFunctionSelected, SubscriptionMessage, UpdateCharts,
};
use crate::views::info_row::{menu_button, INFO_ROW_HEIGHT};
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
use iced::widget::{
    button, horizontal_space, pick_list, row, scrollable, toggler, Button, Column, Row, Text,
};
use iced::widget::{container, Tooltip};
use iced::{Alignment, Center, Element, Length, Size, Task};
use iced::{Color, Renderer, Theme};
use iced_aw::menu::Item;
use iced_aw::{Menu, MenuBar};
use iced_futures::Subscription;
#[cfg(feature = "iroh")]
use iroh::{NodeId, RelayUrl};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tcp")]
use std::net::IpAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const HARDWARE_VIEW_PADDING: f32 = 10.0;

pub(crate) const BOARD_LAYOUT_SIZE: Size = Size {
    width: 1400.0,
    height: 720.0,
};

const SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;

const PIN_ROW_HEIGHT: f32 = 28.0;

// calculate the height required based on the number of configured pins
pub(crate) fn compact_layout_size(num_pins: usize) -> Size {
    let mut height = HARDWARE_VIEW_PADDING
        + (num_pins as f32 * (PIN_ROW_HEIGHT + SPACE_BETWEEN_PIN_ROWS))
        + HARDWARE_VIEW_PADDING
        + INFO_ROW_HEIGHT
        + 1.0;

    // Dock is not shown if pins unconfigured
    if num_pins != 26 {
        height += SPACE_BETWEEN_PIN_ROWS + PIN_ROW_HEIGHT
    }

    Size {
        width: 720.0,
        height,
    }
}

pub(crate) fn bcm_layout_size(num_pins: usize) -> Size {
    Size {
        width: 720.0,
        height: HARDWARE_VIEW_PADDING
            + (num_pins as f32 * (PIN_ROW_HEIGHT + SPACE_BETWEEN_PIN_ROWS))
            + HARDWARE_VIEW_PADDING
            + INFO_ROW_HEIGHT
            + 1.0,
    }
}

/// [HardwareViewMessage] covers all messages that are handled by hardware_view
#[derive(Debug, Clone)]
pub enum HardwareViewMessage {
    Activate(BoardPinNumber),
    PinFunctionSelected(BCMPinNumber, PinFunction),
    NewConfig(HardwareConfig),
    SubscriptionMessage(SubscriptionEvent),
    ChangeOutputLevel(BCMPinNumber, LevelChange),
    UpdateCharts,
}

/// A type of connection to a piece of hardware
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum HardwareConnection {
    NoConnection,
    #[default]
    Local,
    #[cfg(feature = "usb")]
    Usb(SerialNumber),
    #[cfg(feature = "iroh")]
    Iroh(NodeId, Option<RelayUrl>),
    #[cfg(feature = "tcp")]
    Tcp(IpAddr, u16),
}

impl Display for HardwareConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NoConnection => write!(f, "No Connection"),
            Local => write!(f, "Local Hardware"),
            #[cfg(feature = "usb")]
            Usb(_) => write!(f, "USB"),
            #[cfg(feature = "iroh")]
            Iroh(nodeid, _relay_url) => write!(f, "Iroh Network: {nodeid}"),
            #[cfg(feature = "tcp")]
            Tcp(ip, port) => write!(f, "TCP IP:Port: {ip}:{port}"),
        }
    }
}

impl HardwareConnection {
    /// Return a short name describing the connection type
    pub const fn name(&self) -> &'static str {
        match self {
            NoConnection => "disconnected",
            #[cfg(not(target_arch = "wasm32"))]
            Local => "Local",
            #[cfg(feature = "usb")]
            Usb(_) => "USB",
            #[cfg(feature = "iroh")]
            Iroh(_, _) => "Iroh",
            #[cfg(feature = "tcp")]
            Tcp(_, _) => "TCP",
        }
    }
}

pub struct HardwareView {
    hardware_connection: HardwareConnection,
    hardware_config: HardwareConfig,
    subscriber_sender: Option<Sender<SubscriberMessage>>,
    pub hardware_description: Option<HardwareDescription>,
    /// Either desired state of an output, or detected state of input.
    pin_states: HashMap<BCMPinNumber, PinState>,
}

async fn empty() {}

impl HardwareView {
    #[must_use]
    pub fn new(hardware_connection: HardwareConnection) -> Self {
        Self {
            hardware_connection,
            hardware_config: HardwareConfig::default(),
            hardware_description: None, // Until listener is ready
            subscriber_sender: None,    // Until listener is ready
            pin_states: HashMap::new(),
        }
    }

    /// Get the current [HardwareConfig]
    #[must_use]
    pub fn get_config(&self) -> &HardwareConfig {
        &self.hardware_config
    }

    /// Get the current [HardwareConnection]
    #[must_use]
    pub fn get_hardware_connection(&self) -> &HardwareConnection {
        &self.hardware_connection
    }

    /// Returns Some(&str) describing the Model of HW Piggui is connected to, or None
    #[must_use]
    pub fn hw_model(&self) -> Option<&str> {
        self.hardware_description
            .as_ref()
            .map(|desc| desc.details.model.as_str())
    }

    /// Apply the [HardwareConfig] active here to the GPIO hardware
    // TODO this might cause a re-apply of same config coming _from_ the hardware?
    fn update_hw_config(&mut self) {
        if let Some(ref mut subscriber_sender) = &mut self.subscriber_sender {
            let _ = subscriber_sender.try_send(Hardware(HardwareConfigMessage::NewConfig(
                self.hardware_config.clone(),
            )));
        }
    }

    /// Send a message to request the subscription to switch connections to a new one
    pub fn new_connection(&mut self, new_connection: HardwareConnection) {
        self.hardware_description = None;
        self.hardware_connection = new_connection;
        if let Some(ref mut subscription_sender) = &mut self.subscriber_sender {
            let _ = subscription_sender.try_send(SubscriberMessage::NewConnection(
                self.hardware_connection.clone(),
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
            if let Some(ref mut listener) = &mut self.subscriber_sender {
                let _ = listener.try_send(Hardware(HardwareConfigMessage::NewPinConfig(
                    bcm_pin_number,
                    new_function,
                )));
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

    /// Save the new config in the view, update pin states and apply it to the connected hardware
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

            SubscriptionMessage(event) => match event {
                SubscriptionEvent::Connected(hw_desc, hw_config) => {
                    self.hardware_description = Some(hw_desc);
                    self.hardware_config = hw_config;
                    self.set_pin_states_after_load();
                    self.update_hw_config();
                    return Task::perform(empty(), |_| Message::Connected);
                }
                SubscriptionEvent::InputChange(bcm_pin_number, level_change) => {
                    self.pin_states
                        .entry(bcm_pin_number)
                        .or_insert(PinState::new())
                        .set_level(level_change);
                }
                SubscriptionEvent::ConnectionError(error) => {
                    return Task::perform(empty(), move |_| {
                        Message::ConnectionError(error.clone())
                    });
                }
                SubscriptionEvent::Ready(mut subscriber_sender) => {
                    let _ = subscriber_sender.try_send(SubscriberMessage::NewConnection(
                        self.hardware_connection.clone(),
                    ));
                    self.subscriber_sender = Some(subscriber_sender);
                }
            },

            ChangeOutputLevel(bcm_pin_number, level_change) => {
                self.pin_states
                    .entry(bcm_pin_number)
                    .or_insert(PinState::new())
                    .set_level(level_change.clone());
                if let Some(ref mut listener) = &mut self.subscriber_sender {
                    let _ = listener.try_send(Hardware(HardwareConfigMessage::IOLevelChanged(
                        bcm_pin_number,
                        level_change,
                    )));
                }
            }

            Activate(pin_number) => println!("Pin {pin_number} clicked"),
        }

        Task::none()
    }

    /// Construct the view that represents the hardware view
    pub fn view(&self, layout: Layout) -> Element<Message> {
        let inner: Element<HardwareViewMessage> =
            if let Some(hw_description) = &self.hardware_description {
                let pin_layout = match layout {
                    Layout::Board => self.board_pin_layout_view(&hw_description.pins),
                    Layout::Logical => self.bcm_pin_layout_view(&hw_description.pins),
                    Layout::Compact => self.compact_layout_view(&hw_description.pins),
                };

                scrollable(pin_layout)
                    .direction({
                        let scrollbar = Scrollbar::new().width(10);
                        scrollable::Direction::Both {
                            horizontal: scrollbar,
                            vertical: scrollbar,
                        }
                    })
                    .into()
            } else {
                // The no hardware view will go here and maybe some widget to search for and connect to remote HW?
                Row::new().into()
            };

        let hw_column = Column::new()
            .push(inner.map(Message::Hardware))
            .align_x(Center)
            .height(Length::Fill)
            .width(Length::Fill);

        container(hw_column).padding(HARDWARE_VIEW_PADDING).into()
    }

    /// Create subscriptions for ticks for updating charts of waveforms and events coming from hardware
    pub fn subscription(&self) -> Subscription<HardwareViewMessage> {
        let subscriptions = vec![
            iced::time::every(Duration::from_millis(1000 / CHART_UPDATES_PER_SECOND))
                .map(|_| UpdateCharts),
            Subscription::run_with_id("hardware", hardware_subscription::subscribe())
                .map(SubscriptionMessage),
        ];

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

    /// Compact view that only lays out configured pins
    pub fn compact_layout_view<'a>(
        &'a self,
        pin_set: &'a PinDescriptionSet,
    ) -> Element<'a, HardwareViewMessage> {
        let mut column: Column<HardwareViewMessage> =
            Column::new().width(Length::Shrink).height(Length::Shrink);

        // add a row at the top that is a "dock" for unconfigured pins
        let mut unused_pins: Vec<Item<'a, HardwareViewMessage, Theme, Renderer>> = vec![];
        for pin_description in pin_set.pins() {
            if let Some(bcm_pin_number) = &pin_description.bcm {
                if !self
                    .hardware_config
                    .pin_functions
                    .contains_key(bcm_pin_number)
                {
                    // Pin is not configured, add it to the doc, just as a pin
                    unused_pins.push(pin_button(
                        pin_description.bpn,
                        pin_description.name.as_ref(),
                    ));
                }
            }
        }

        if !unused_pins.is_empty() {
            let pin_dock: MenuBar<HardwareViewMessage, Theme, Renderer> = MenuBar::new(unused_pins);
            column = column.push(pin_dock);
        }

        // Add a row for each configured pin
        for pin_description in pin_set.bcm_pins_sorted() {
            if let Some(bcm_pin_number) = &pin_description.bcm {
                if self
                    .hardware_config
                    .pin_functions
                    .contains_key(bcm_pin_number)
                {
                    // Pin is configured, lay it out with full widgets in a row
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

        Some(Output(level)) => {
            let output_toggler = toggler(
                pin_state
                    .get_level()
                    .unwrap_or(level.unwrap_or(false as PinLevel)),
            )
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

/// Filter the selected_option out of the list of other selectable options for the PickList
/// The options returned should be generic as to the sub-options. i.e. pullup/pulldown/none for an
/// [Input] or true/false for an [Output], as those sub-selections are taken care of by other
/// widgets
fn filter_options(
    options: &[PinFunction],
    selected_function: Option<PinFunction>,
) -> Vec<PinFunction> {
    let mut config_options: Vec<_> = options
        .iter()
        .filter(|&&option| match selected_function {
            Some(Input(_)) => {
                matches!(option, Output(_) | PinFunction::None)
            }
            Some(Output(_)) => {
                matches!(option, Input(_) | PinFunction::None)
            }
            Some(selected) => selected != option,
            None => option != PinFunction::None,
        })
        .map(|option| match option {
            Input(_) => Input(None),
            Output(_) => Output(None),
            PinFunction::None => PinFunction::None,
        })
        .collect();

    // Always ensure there is a [PinFunction::None] option present
    if !config_options.contains(&PinFunction::None)
        && selected_function.is_some()
        && selected_function != Some(PinFunction::None)
    {
        config_options.push(PinFunction::None);
    }

    config_options
}

/// Create a button representing the pin with its physical (bpn) number, color and a menu
fn pin_button(
    bpn: BoardPinNumber,
    pin_name: &str,
) -> Item<'_, HardwareViewMessage, Theme, Renderer> {
    let button = button(
        container(Text::new(bpn.to_string()))
            .align_x(Center)
            .align_y(Center),
    )
    .padding(0.0)
    .width(Length::Fixed(PIN_BUTTON_WIDTH))
    .height(Length::Fixed(PIN_BUTTON_WIDTH))
    .style(move |_, _| get_pin_style(pin_name));

    let mut menu_items: Vec<Item<'_, HardwareViewMessage, _, _>> = vec![];

    let bla = Button::new("Bl bla").width(Length::Fill).style(menu_button);

    menu_items.push(Item::new(bla));

    Item::with_menu(button, Menu::new(menu_items).width(135.0).offset(10.0))
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

        // Filter options to remove currently selected one
        let config_options = filter_options(&pin_description.options, pin_function.cloned());

        if !config_options.is_empty() {
            let selected = pin_function.filter(|&pin_function| pin_function != &PinFunction::None);

            let pick_list = pick_list(config_options, selected, move |pin_function| {
                PinFunctionSelected(bcm_pin_number, pin_function)
            })
            .width(Length::Fixed(PIN_OPTION_WIDTH))
            .placeholder("Select function");

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

    let pin_button: Element<HardwareViewMessage> = MenuBar::new(vec![pin_button(
        pin_description.bpn,
        pin_description.name.as_ref(),
    )])
    .into();

    // Create the row of widgets that represent the pin, inverted order if left or right
    let row = if direction == Left {
        row![
            pin_widget,
            pin_option,
            pin_name_column.align_x(Alignment::End),
            pin_arrow,
            pin_button
        ]
    } else {
        row![
            pin_button,
            pin_arrow,
            pin_name_column.align_x(Alignment::Start),
            pin_option,
            pin_widget
        ]
    };

    row.align_y(Center).spacing(WIDGET_ROW_SPACING)
}

#[cfg(test)]
mod test {
    use crate::hw_definition::config::InputPull::{PullDown, PullUp};
    use crate::views::hardware_view::HardwareConnection::NoConnection;
    use crate::views::hardware_view::HardwareView;

    #[test]
    fn no_hardware_description() {
        let hw_view = HardwareView::new(NoConnection);
        assert!(hw_view.hardware_description.is_none());
    }

    #[test]
    fn no_hardware_model() {
        let hw_view = HardwareView::new(NoConnection);
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

        // Test case: Input selected
        let result = filter_options(&options, Some(Input(Some(PullUp))));
        assert_eq!(result, vec![Output(None), PinFunction::None]);

        // Test case: Input selected
        let result = filter_options(&options, Some(Input(Some(PullDown))));
        assert_eq!(result, vec![Output(None), PinFunction::None]);

        // Test case: Output selected
        let result = filter_options(&options, Some(Output(None)));
        assert_eq!(result, vec![Input(None), PinFunction::None]);

        // Test case: Output with value selected
        let result = filter_options(&options, Some(Output(Some(true))));
        assert_eq!(result, vec![Input(None), PinFunction::None]);

        // Test case: Output with value selected
        let result = filter_options(&options, Some(Output(Some(false))));
        assert_eq!(result, vec![Input(None), PinFunction::None]);

        // Test case: None selected
        let result = filter_options(&options, Some(PinFunction::None));
        assert_eq!(result, vec![Input(None), Output(None)]);
    }

    // Test the filter option when the inputs are not generic, but have sub-selections
    #[test]
    fn test_other_filter_options() {
        use super::*;

        let options = vec![Input(Some(PullDown)), Output(None)];

        let result = filter_options(&options, Some(Output(Some(true))));
        assert_eq!(result, vec![Input(None), PinFunction::None]);
    }
}
