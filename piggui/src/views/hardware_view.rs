use crate::hardware_subscription;
use crate::hardware_subscription::SubscriberMessage::Hardware;
use crate::hardware_subscription::{SubscriberMessage, SubscriptionEvent};
use crate::views::hardware_styles::{get_pin_style, toggler_style, TOOLTIP_STYLE};
use crate::views::hardware_view::HardwareViewMessage::{
    Activate, ChangeOutputLevel, MenuBarButtonClicked, NewConfig, PinFunctionChanged,
    SubscriptionMessage, UpdateCharts,
};
use crate::views::info_row::{menu_button_style, INFO_ROW_HEIGHT};
use crate::views::layout_menu::Layout;
use crate::views::pin_state::{PinState, CHART_UPDATES_PER_SECOND, CHART_WIDTH};
use crate::widgets::led::led;
use crate::widgets::{circle::circle, line::line};
use crate::Message;
use iced::advanced::text::editor::Direction::{Left, Right};
use iced::futures::channel::mpsc::Sender;
use iced::widget::scrollable::Scrollbar;
use iced::widget::tooltip::Position;
use iced::widget::{
    button, horizontal_space, row, scrollable, text, toggler, Button, Column, Row, Text,
};
use iced::widget::{container, Tooltip};
use iced::Alignment::{End, Start};
use iced::{alignment, Alignment, Center, Element, Fill, Length, Size, Task};
use iced::{Renderer, Theme};
use iced_aw::menu::Item;
use iced_aw::{Menu, MenuBar};
use iced_futures::Subscription;
use pigdef::config::InputPull::{PullDown, PullUp};
use pigdef::config::LevelChange;
use pigdef::config::{HardwareConfig, HardwareConfigMessage};
use pigdef::description::{BCMPinNumber, BoardPinNumber, PinLevel};
use pigdef::description::{HardwareDescription, PinDescription, PinDescriptionSet};
use pigdef::pin_function::PinFunction;
use pigdef::pin_function::PinFunction::{Input, Output};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pignet::HardwareConnection;
use pignet::HardwareConnection::Local;

const HARDWARE_VIEW_PADDING: f32 = 10.0;
const PIN_DOCK_SPACING: f32 = 2.0;

const SPACE_BETWEEN_PIN_COLUMNS: f32 = 10.0;

const SPACE_BETWEEN_PIN_ROWS: f32 = 5.0;

const MAX_CONFIGURABLE_PINS: usize = 26;

// WIDTHS
const PIN_BUTTON_DIAMETER: f32 = 28.0;
pub(crate) const PIN_BUTTON_RADIUS: f32 = PIN_BUTTON_DIAMETER / 2.0;
const PIN_ARROW_LINE_WIDTH: f32 = 20.0;
const PIN_ARROW_CIRCLE_RADIUS: f32 = 5.0;
const PIN_ROW_WIDTH: f32 =
    PIN_ARROW_LINE_WIDTH + (PIN_ARROW_CIRCLE_RADIUS * 2.0) + PIN_BUTTON_DIAMETER;
const PIN_NAME_WIDTH: f32 = 80.0; // for some longer pin names
const TOGGLER_SIZE: f32 = 28.0;
const TOGGLER_WIDTH: f32 = 65.0;
const WIDGET_ROW_SPACING: f32 = 5.0;
const PIN_WIDGET_ROW_WIDTH: f32 =
    (LED_RADIUS * 2.0) + WIDGET_ROW_SPACING + CHART_WIDTH + WIDGET_ROW_SPACING + TOGGLER_WIDTH;

const LED_RADIUS: f32 = 14.0;

pub(crate) const fn board_layout_size(_number_of_pins: usize) -> Size {
    Size {
        width: 1060.0,
        height: 720.0,
    }
}

// calculate the height required based on the number of configured pins
pub(crate) fn compact_layout_size(num_configured_pins: usize) -> Size {
    let mut height = HARDWARE_VIEW_PADDING
        + (num_configured_pins as f32 * (PIN_BUTTON_DIAMETER + SPACE_BETWEEN_PIN_ROWS))
        + HARDWARE_VIEW_PADDING
        + INFO_ROW_HEIGHT
        + 1.0;

    // Dock is not shown if pins unconfigured
    if num_configured_pins != MAX_CONFIGURABLE_PINS {
        height += SPACE_BETWEEN_PIN_ROWS + PIN_BUTTON_DIAMETER
    }

    let num_unconfigured_pins = 26 - num_configured_pins;

    Size {
        width: HARDWARE_VIEW_PADDING
            + (num_unconfigured_pins as f32 * (PIN_BUTTON_DIAMETER + PIN_DOCK_SPACING))
            + HARDWARE_VIEW_PADDING,
        height,
    }
}

pub(crate) fn bcm_layout_size(num_pins: usize) -> Size {
    Size {
        width: 540.0,
        height: HARDWARE_VIEW_PADDING
            + (num_pins as f32 * (PIN_BUTTON_DIAMETER + SPACE_BETWEEN_PIN_ROWS))
            + HARDWARE_VIEW_PADDING
            + INFO_ROW_HEIGHT
            + 1.0,
    }
}

/// [HardwareViewMessage] covers all messages that are handled by hardware_view
#[derive(Debug, Clone)]
pub enum HardwareViewMessage {
    Activate(BoardPinNumber),
    PinFunctionChanged(BCMPinNumber, Option<PinFunction>, bool, bool),
    NewConfig(HardwareConfig),
    SubscriptionMessage(SubscriptionEvent),
    ChangeOutputLevel(BCMPinNumber, LevelChange),
    UpdateCharts,
    MenuBarButtonClicked, // needed for highlighting to work
}

pub struct HardwareView {
    hardware_connection: HardwareConnection,
    hardware_config: HardwareConfig,
    subscriber_sender: Option<Sender<SubscriberMessage>>,
    hardware_description: Option<HardwareDescription>,
    /// Either the desired state of an output or the detected state of input
    pin_states: HashMap<BCMPinNumber, PinState>,
}

async fn empty() {}

impl HardwareView {
    #[must_use]
    pub fn new(hardware_connection: HardwareConnection) -> Self {
        Self {
            hardware_connection,
            hardware_config: HardwareConfig::default(),
            hardware_description: None, // Until the listener is ready
            subscriber_sender: None,    // Until the listener is ready
            pin_states: HashMap::new(),
        }
    }

    /// Get the current [HardwareConfig]
    #[must_use]
    pub fn get_config(&self) -> &HardwareConfig {
        &self.hardware_config
    }

    /// Get the current [HardwareDescription]
    #[must_use]
    pub fn get_description(&self) -> &Option<HardwareDescription> {
        &self.hardware_description
    }

    /// Get the current [HardwareConnection]
    #[must_use]
    pub fn get_hardware_connection(&self) -> &HardwareConnection {
        &self.hardware_connection
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
    fn new_pin_function(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        new_function: Option<PinFunction>,
        resize_window: bool,
        mark_unsaved: bool,
    ) -> Task<Message> {
        let previous_function = self.hardware_config.pin_functions.get(&bcm_pin_number);

        if new_function.as_ref() != previous_function {
            match new_function {
                None => {
                    self.hardware_config.pin_functions.remove(&bcm_pin_number);
                    self.pin_states.remove(&bcm_pin_number);
                }
                Some(function) => {
                    self.hardware_config
                        .pin_functions
                        .insert(bcm_pin_number, function);
                    self.pin_states.insert(bcm_pin_number, PinState::new());
                }
            }

            // Report config changes to the hardware listener
            // Since config loading and hardware listener setup can occur out-of-order
            // mark the config as changed. If we send to the listener, then mark as done
            if let Some(ref mut listener) = &mut self.subscriber_sender {
                let _ = listener.try_send(Hardware(HardwareConfigMessage::NewPinConfig(
                    bcm_pin_number,
                    new_function,
                )));
            }
            return Task::perform(empty(), move |_| {
                Message::ConfigChangesMade(resize_window, mark_unsaved)
            });
        }

        Task::none()
    }

    /// Save the new config in the view, update pin states and apply it to the connected hardware
    pub fn new_config(&mut self, new_config: HardwareConfig) {
        self.hardware_config = new_config;
        self.set_pin_states_after_load();
        self.update_hw_config();
    }

    /// Go through all the pins in the [HardwareConfig], make sure a pin state exists for the pin
    /// and then set the current level if pin is an Output and the level was specified.
    /// TODO check if still needed - or should we add level reading to apply_config_change()
    /// TODO at the end of hardware_subscription.rs and keep this out of the UI code.
    /// TODO that should take care of states for outputs also. If we move that in there, then
    /// TODO it should use hardware::get_time_since_boot()
    fn set_pin_states_after_load(&mut self) {
        for (bcm_pin_number, pin_function) in &self.hardware_config.pin_functions {
            // For output pins, if there is an initial state set, then set that in the pin state
            // so the toggler will be drawn correctly on the first draw
            if let Output(Some(level)) = pin_function {
                if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    self.pin_states
                        .entry(*bcm_pin_number)
                        .or_insert(PinState::new())
                        .set_level(LevelChange::new(*level, now));
                }
            }
        }
    }

    pub fn update(&mut self, message: HardwareViewMessage) -> Task<Message> {
        match message {
            UpdateCharts => {
                // Update all the charts of the pins that have an assigned function
                for pin in self.pin_states.values_mut() {
                    pin.chart.refresh();
                }
            }

            PinFunctionChanged(bcm_pin_number, pin_function, resize_window, mark_unsaved) => {
                return self.new_pin_function(
                    bcm_pin_number,
                    pin_function,
                    resize_window,
                    mark_unsaved,
                );
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
            MenuBarButtonClicked => { /* For highlighting */ }
        }

        Task::none()
    }

    /// Construct the hardware view
    pub fn view(&self, layout: Layout) -> Element<'_, Message> {
        let inner: Element<HardwareViewMessage> =
            if let Some(hw_description) = &self.hardware_description {
                let pin_layout = match layout {
                    Layout::Board => self.board_pin_layout_view(&hw_description.pins),
                    Layout::Logical => self.bcm_pin_layout_view(&hw_description.pins),
                    Layout::Compact => self.compact_layout_view(&hw_description.pins),
                };

                scrollable(pin_layout)
                    .direction({
                        let scrollbar = Scrollbar::new().width(HARDWARE_VIEW_PADDING);
                        scrollable::Direction::Both {
                            horizontal: scrollbar,
                            vertical: scrollbar,
                        }
                    })
                    .width(Fill)
                    .height(Fill)
                    .into()
            } else {
                Self::empty_layout_view()
            };

        container(inner.map(Message::Hardware))
            .padding(HARDWARE_VIEW_PADDING)
            .into()
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

    /// Draw an empty view when there is no connection to hardware
    pub fn empty_layout_view<'a>() -> Element<'a, HardwareViewMessage> {
        let col = Column::new()
            .width(Fill)
            .push(text("Disconnected").size(20))
            .push(text("You are not currently connected to any device with GPIO hardware to display"))
            .push(text("You can use the 'device' menu at the bottom of the window to connect to detected devices"))
            .push(text("If you know the TCP or Iroh connection data, you can use the 'disconnected:' menu and the 'Connect to remote Pi ...' option"))
            .spacing(SPACE_BETWEEN_PIN_ROWS)
            .align_x(Center);

        Row::new()
            .width(Fill)
            .height(Fill)
            .push(col)
            .align_y(Center)
            .into()
    }

    /// View that lays out the pins in a single column ordered by BCM pin number
    pub fn bcm_pin_layout_view<'a>(
        &'a self,
        pin_set: &'a PinDescriptionSet,
    ) -> Element<'a, HardwareViewMessage> {
        let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

        for pin_description in pin_set.bcm_pins_sorted() {
            if let Some(bcm_pin_number) = &pin_description.bcm {
                if let Some(bcm) = pin_description.bcm {
                    let pin_row = self.create_pin_view_side(
                        pin_description,
                        self.hardware_config.pin_functions.get(&bcm),
                        Start,
                        self.pin_states.get(bcm_pin_number),
                        false,
                    );

                    column = column.push(pin_row);
                }
            }
        }

        column.spacing(SPACE_BETWEEN_PIN_ROWS).align_x(Start).into()
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
                    unused_pins.push(self.pin_button_menu(
                        pin_description,
                        self.hardware_config.pin_functions.get(bcm_pin_number),
                        true,
                    ));
                }
            }
        }

        if !unused_pins.is_empty() {
            let pin_dock: MenuBar<HardwareViewMessage, Theme, Renderer> =
                MenuBar::new(unused_pins).spacing(PIN_DOCK_SPACING);
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
                    if let Some(bcm) = pin_description.bcm {
                        // Pin is configured, lay it out with full widgets in a row
                        let pin_row = self.create_pin_view_side(
                            pin_description,
                            self.hardware_config.pin_functions.get(&bcm),
                            Start,
                            self.pin_states.get(bcm_pin_number),
                            true,
                        );

                        column = column.push(pin_row);
                    }
                }
            }
        }

        column.spacing(SPACE_BETWEEN_PIN_ROWS).align_x(Start).into()
    }

    /// View that draws the pins laid out as they are on the physical Pi board
    pub fn board_pin_layout_view<'a>(
        &'a self,
        pin_descriptions: &'a PinDescriptionSet,
    ) -> Element<'a, HardwareViewMessage> {
        let mut column = Column::new().width(Length::Shrink).height(Length::Shrink);

        // Draw all pins, those with and without [BCMPinNumber]
        for pair in pin_descriptions.pins().chunks(2) {
            let left_row = self.create_pin_view_side(
                &pair[0],
                pair[0]
                    .bcm
                    .and_then(|bcm| self.hardware_config.pin_functions.get(&bcm)),
                End,
                pair[0].bcm.and_then(|bcm| self.pin_states.get(&bcm)),
                false,
            );

            let right_row = self.create_pin_view_side(
                &pair[1],
                pair[1]
                    .bcm
                    .and_then(|bcm| self.hardware_config.pin_functions.get(&bcm)),
                Start,
                pair[1].bcm.and_then(|bcm| self.pin_states.get(&bcm)),
                false,
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

    /// Create a row of widgets that represent a pin, either from left to right or right to left
    fn create_pin_view_side<'a>(
        &self,
        pin_description: &'a PinDescription,
        pin_function: Option<&'a PinFunction>,
        alignment: Alignment,
        pin_state: Option<&'a PinState>,
        resize_window_on_change: bool,
    ) -> Row<'a, HardwareViewMessage> {
        let pin_widget = if let Some(state) = pin_state {
            // Create a widget used either to visualize an input or control an output
            get_pin_widget(pin_description.bcm, pin_function, state, alignment)
        } else {
            horizontal_space().width(PIN_WIDGET_ROW_WIDTH).into()
        };

        let pin_name = Text::new(&pin_description.name)
            .width(PIN_NAME_WIDTH)
            .align_x(alignment);

        let mut pin_row = Row::new().align_y(Center).width(PIN_ROW_WIDTH);

        // If the pin is configurable, create a menu on it, if not just the button
        let pin_button: Element<HardwareViewMessage> = if let Some(bcm) = pin_description.bcm {
            MenuBar::new(vec![self.pin_button_menu(
                pin_description,
                self.hardware_config.pin_functions.get(&bcm),
                resize_window_on_change,
            )])
            .style(|_, _| crate::views::info_row::MENU_BAR_STYLE)
            .into()
        } else {
            pin_button(pin_description).into()
        };

        // Create the row of widgets that represent the pin, inverted order if left or right
        let row = if alignment == End {
            pin_row = pin_row.push(circle(PIN_ARROW_CIRCLE_RADIUS));
            pin_row = pin_row.push(line(PIN_ARROW_LINE_WIDTH));
            pin_row = pin_row.push(pin_button);
            row![pin_widget, pin_name, pin_row,]
        } else {
            pin_row = pin_row.push(pin_button);
            pin_row = pin_row.push(line(PIN_ARROW_LINE_WIDTH));
            pin_row = pin_row.push(circle(PIN_ARROW_CIRCLE_RADIUS));
            row![pin_row, pin_name, pin_widget]
        };

        row.align_y(Center).spacing(WIDGET_ROW_SPACING)
    }

    /// Create a button representing the pin with its physical (bpn) number, color and maybe a menu
    fn pin_button_menu<'a>(
        &self,
        pin_description: &'a PinDescription,
        current_option: Option<&PinFunction>,
        resize_window_on_change: bool,
    ) -> Item<'a, HardwareViewMessage, Theme, Renderer> {
        let mut pin_menu_items: Vec<Item<HardwareViewMessage, _, _>> = vec![];
        if let Some(bcm_pin_number) = pin_description.bcm {
            for option in pin_description.options.iter() {
                match option {
                    Input(_) => {
                        let mut pullup_items = vec![];
                        for (name, pullup) in [
                            ("Pullup", Some(PullUp)),
                            ("Pulldown", Some(PullDown)),
                            ("None", None),
                        ] {
                            let mut pullup_button =
                                button(name).width(Fill).style(menu_button_style);
                            if let Some(&Input(pull)) = current_option {
                                if pullup != pull {
                                    pullup_button = pullup_button.on_press(PinFunctionChanged(
                                        bcm_pin_number,
                                        Some(Input(pullup)),
                                        resize_window_on_change,
                                        self.hardware_connection != Local,
                                    ));
                                }
                            } else {
                                pullup_button = pullup_button.on_press(PinFunctionChanged(
                                    bcm_pin_number,
                                    Some(Input(pullup)),
                                    resize_window_on_change,
                                    self.hardware_connection != Local,
                                ));
                            }
                            pullup_items.push(Item::new(pullup_button));
                        }
                        let input_button = button(row!(
                            text("Input"),
                            horizontal_space(),
                            text(" >").align_y(alignment::Vertical::Center),
                        ))
                        .width(100.0)
                        .on_press(MenuBarButtonClicked) // Needed for highlighting
                        .style(menu_button_style);
                        pin_menu_items.push(Item::with_menu(
                            input_button,
                            Menu::new(pullup_items).width(80.0),
                        ));
                    }

                    Output(_) => {
                        let mut output_button =
                            button("Output").width(Fill).style(menu_button_style);
                        if !matches!(current_option, Some(&Output(..))) {
                            output_button = output_button.on_press(PinFunctionChanged(
                                bcm_pin_number,
                                Some(Output(None)),
                                resize_window_on_change,
                                self.hardware_connection != Local,
                            ));
                        }
                        pin_menu_items.push(Item::new(output_button));
                    }
                }
            }

            let mut unused = button("Unused").width(Fill).style(menu_button_style);
            if current_option.is_some() {
                unused = unused.on_press(PinFunctionChanged(
                    bcm_pin_number,
                    None,
                    resize_window_on_change,
                    self.hardware_connection != Local,
                ));
            }
            pin_menu_items.push(Item::new(unused));
        }

        Item::with_menu(
            pin_button(pin_description).on_press(MenuBarButtonClicked), // Needed for highlighting
            Menu::new(pin_menu_items).width(80.0),
        )
    }
}

/// Create the widget that either shows an input pin's state
/// or allows the user to control the state of an output pin
/// This should only be called for pins that have a valid BCMPinNumber
fn get_pin_widget<'a>(
    bcm_pin_number: Option<BCMPinNumber>,
    pin_function: Option<&'a PinFunction>,
    pin_state: &'a PinState,
    alignment: Alignment,
) -> Element<'a, HardwareViewMessage> {
    let row: Row<HardwareViewMessage> = match pin_function {
        Some(Input(_)) => {
            let led = led(LED_RADIUS, pin_state.get_level());
            if alignment == End {
                Row::new()
                    .push(pin_state.view(Left))
                    .push(led)
                    .push(horizontal_space().width(TOGGLER_WIDTH))
            } else {
                Row::new()
                    .push(horizontal_space().width(TOGGLER_WIDTH))
                    .push(led)
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
                if let Some(bcm) = bcm_pin_number {
                    if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                        ChangeOutputLevel(bcm, LevelChange::new(b, now))
                    } else {
                        MenuBarButtonClicked // Fake message in case of error
                    }
                } else {
                    MenuBarButtonClicked // Fake message in case of error
                }
            })
            .size(TOGGLER_SIZE)
            .style(toggler_style);

            let toggle_tooltip =
                Tooltip::new(output_toggler, "Click to toggle level", Position::Top)
                    .gap(4.0)
                    .style(|_| TOOLTIP_STYLE);

            let toggle_level_message = if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH)
            {
                if let Some(bcm) = bcm_pin_number {
                    let level: PinLevel = pin_state.get_level().unwrap_or(false as PinLevel);
                    ChangeOutputLevel(bcm, LevelChange::new(!level, now))
                } else {
                    MenuBarButtonClicked // Fake for the error case
                }
            } else {
                MenuBarButtonClicked // Fake for the error case
            };

            let led = led::<HardwareViewMessage>(LED_RADIUS, pin_state.get_level())
                .on_press(toggle_level_message.clone())
                .on_release(toggle_level_message);

            let led_tooltip = Tooltip::new(led, "Hold down to invert level", Position::Top)
                .gap(4.0)
                .style(|_| TOOLTIP_STYLE);

            if alignment == End {
                Row::new()
                    .push(pin_state.view(Left))
                    .push(led_tooltip)
                    .push(toggle_tooltip)
            } else {
                Row::new()
                    .push(toggle_tooltip)
                    .push(led_tooltip)
                    .push(pin_state.view(Right))
            }
        }

        _ => Row::new(),
    };

    row.width(PIN_WIDGET_ROW_WIDTH)
        .spacing(WIDGET_ROW_SPACING)
        .align_y(Center)
        .into()
}

/// Create a button representing the pin with its physical (bpn) number, color
fn pin_button(pin_description: &PinDescription) -> Button<'_, HardwareViewMessage> {
    button(
        container(text(pin_description.bpn))
            .align_x(Center)
            .align_y(Center),
    )
    .padding(0.0)
    .width(PIN_BUTTON_DIAMETER)
    .height(PIN_BUTTON_DIAMETER)
    .style(move |_, status| {
        get_pin_style(
            status,
            pin_description.name.as_ref(),
            !pin_description.options.is_empty(),
        )
    })
}

#[cfg(test)]
mod test {
    use crate::views::hardware_view::HardwareConnection::NoConnection;
    use crate::views::hardware_view::HardwareView;

    #[test]
    fn no_hardware_description() {
        let hw_view = HardwareView::new(NoConnection);
        assert!(hw_view.hardware_description.is_none());
    }
}
