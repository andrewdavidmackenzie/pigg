use iced::event::{self, Event};
use std::{env, io};

use iced::futures::channel::mpsc::Sender;
use iced::widget::{self, container, pick_list, Button, Column, Row, Text};
use iced::{
    alignment, executor, window, Alignment, Application, Color, Command, Element, Length, Settings,
    Size, Subscription, Theme,
};
use ringbuffer::{AllocRingBuffer, RingBuffer};

use hw::HardwareDescriptor;
use hw::InputPull;
use hw::{BCMPinNumber, BoardPinNumber, GPIOConfig, PinFunction, PinLevel};
use hw_listener::{HWListenerEvent, HardwareEvent};
use pin_layout::{bcm_pin_layout_view, board_pin_layout_view};
use style::CustomButton;
use toast::Toast;

// Importing pin layout views
use crate::hw::{LevelChange, PinDescriptionSet};

mod custom_widgets;
mod hw;
mod hw_listener;
mod pin_layout;
mod style;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = "piggui";
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const BOARD_LAYOUT_SPACING: u16 = 470;
const BCM_LAYOUT_SPACING: u16 = 640;
const BOARD_LAYOUT_SIZE: (f32, f32) = (1100.0, 780.0);
const BCM_LAYOUT_SIZE: (f32, f32) = (800.0, 950.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    BoardLayout,
    BCMLayout,
}

impl Layout {
    const ALL: [Layout; 2] = [Layout::BoardLayout, Layout::BCMLayout];

    fn get_spacing(&self) -> u16 {
        match self {
            Layout::BoardLayout => BOARD_LAYOUT_SPACING,
            Layout::BCMLayout => BCM_LAYOUT_SPACING,
        }
    }
}

#[derive(Clone)]
/// PinState captures the state of a pin, including a history of previous states set/read
pub struct PinState {
    history: AllocRingBuffer<LevelChange>,
}

impl PinState {
    /// Create a new PinState with an empty history of states
    fn new() -> Self {
        PinState {
            history: AllocRingBuffer::with_capacity_power_of_2(7),
        }
    }

    /// Try and get the last reported level of the pin, which could be considered "current level"
    /// if everything is working correctly.
    pub fn get_level(&self) -> Option<PinLevel> {
        self.history
            .back()
            .map(|level_change| level_change.new_level)
    }

    /// Add a LevelChange to the history of this pin's state
    pub fn set_level(&mut self, level_change: LevelChange) {
        self.history.push(level_change);
    }
}

// Implementing format for Layout
// TODO could maybe put the Name as a &str inside the enum elements above?
impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layout::BoardLayout => "Board Pin Layout",
                Layout::BCMLayout => "BCM Pin Layout",
            }
        )
    }
}

#[must_use]
pub fn version() -> String {
    format!(
        "{name} {version}\n\
        Copyright (C) 2024 The {name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {name} Contributors.\n\
        Full source available at: {repository}",
        name = NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
        repository = REPOSITORY,
    )
}

fn main() -> Result<(), iced::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("{}", version());
        return Ok(());
    }

    let size = Gpio::get_dimensions_for_layout(Layout::BoardLayout);
    let window = window::Settings {
        resizable: true,
        size,
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    Activate(BoardPinNumber),
    PinFunctionSelected(BoardPinNumber, BCMPinNumber, PinFunction),
    LayoutChanged(Layout),
    ConfigLoaded((String, GPIOConfig)),
    None,
    HardwareListener(HWListenerEvent),
    ChangeOutputLevel(LevelChange),
    Add,
    Close(usize),
    Timeout(f64),
    Event(Event),
    Save,
    Load,
}

pub struct Gpio {
    #[allow(dead_code)]
    config_filename: Option<String>,
    gpio_config: GPIOConfig,
    pub pin_function_selected: [PinFunction; 40],
    chosen_layout: Layout,
    hardware_description: Option<HardwareDescriptor>,
    listener_sender: Option<Sender<HardwareEvent>>,
    /// Either desired state of an output, or detected state of input.
    /// Note: Indexed by BoardPinNumber -1 (since BoardPinNumbers start at 1)
    pin_states: [PinState; 40],
    pin_descriptions: Option<PinDescriptionSet>,
    toasts: Vec<Toast>,
    timeout_secs: u64,
}

impl Gpio {
    async fn load(filename: Option<String>) -> io::Result<Option<(String, GPIOConfig)>> {
        match filename {
            Some(config_filename) => {
                let config = GPIOConfig::load(&config_filename)?;
                Ok(Some((config_filename, config)))
            }
            None => Ok(None),
        }
    }

    async fn load_via_picker() -> io::Result<Option<(String, GPIOConfig)>> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .set_title("Choose config file to load")
            .set_directory(env::current_dir().unwrap())
            .pick_file()
            .await
        {
            let path: std::path::PathBuf = handle.path().to_owned();
            let path_str = path.display().to_string();
            Self::load(Some(path_str)).await
        } else {
            Ok(None)
        }
    }

    async fn save_via_picker(gpio_config: GPIOConfig) -> io::Result<()> {
        if let Some(handle) = rfd::AsyncFileDialog::new()
            .set_title("Choose file")
            .set_directory(env::current_dir().unwrap())
            .save_file()
            .await
        {
            let path: std::path::PathBuf = handle.path().to_owned();
            let path_str = path.display().to_string();
            gpio_config.save(&path_str)
        } else {
            Ok(())
        }
    }

    /// Send the GPIOConfig from the GUI to the hardware to have it applied
    fn update_hw_config(&mut self) {
        if let Some(ref mut listener) = &mut self.listener_sender {
            let _ = listener.try_send(HardwareEvent::NewConfig(self.gpio_config.clone()));
        }
    }

    /// A new function has been selected for a pin via the UI, this function:
    /// - updates the pin_selected_function array for the UI
    /// - saves it in the gpio_config, so when we save later it's there
    /// - sends the update to the hardware to have it applied
    fn new_pin_function(
        &mut self,
        board_pin_number: BoardPinNumber,
        bcm_pin_number: BCMPinNumber,
        new_function: PinFunction,
    ) {
        let pin_index = board_pin_number as usize - 1;
        let previous_function = self.pin_function_selected[pin_index];
        if new_function != previous_function {
            self.pin_function_selected[pin_index] = new_function;
            // Pushing selected pin to the Pin Config
            if let Some(pin_config) = self
                .gpio_config
                .configured_pins
                .iter_mut()
                .find(|(pin, _)| *pin == bcm_pin_number)
            {
                *pin_config = (bcm_pin_number, new_function);
            } else {
                // TODO this could just be adding to the config, not replacing existing ones, no?
                // Add a new configuration entry if it doesn't exist
                self.gpio_config
                    .configured_pins
                    .push((bcm_pin_number, new_function));
            }
            // Report config changes to the hardware listener
            // Since config loading and hardware listener setup can occur out of order
            // mark the config as changed. If we send to the listener, then mark as done
            if let Some(ref mut listener) = &mut self.listener_sender {
                let _ =
                    listener.try_send(HardwareEvent::NewPinConfig(bcm_pin_number, new_function));
            }
        }
    }

    /// Go through all the pins in the loaded GPIOConfig and set its function in the
    /// pin_function_selected array, which is what is used for drawing the UI correctly.
    fn set_pin_functions_after_load(&mut self) {
        if let Some(pin_set) = &self.pin_descriptions {
            for (bcm_pin_number, function) in &self.gpio_config.configured_pins {
                if let Some(board_pin_number) = pin_set.bcm_to_board(*bcm_pin_number) {
                    self.pin_function_selected[board_pin_number as usize - 1] = *function;

                    // For output pins, if there is an initial state set then set that in pin state
                    // so the toggler will be drawn correctly on first draw
                    if let PinFunction::Output(Some(level)) = function {
                        self.pin_states[board_pin_number as usize - 1]
                            .set_level(LevelChange::new(*bcm_pin_number, *level));
                    }
                }
            }
        }
    }
    fn get_dimensions_for_layout(layout: Layout) -> Size {
        match layout {
            Layout::BoardLayout => Size {
                width: BOARD_LAYOUT_SIZE.0,
                height: BOARD_LAYOUT_SIZE.1,
            },
            Layout::BCMLayout => Size {
                width: BCM_LAYOUT_SIZE.0,
                height: BCM_LAYOUT_SIZE.1,
            },
        }
    }
}

impl Application for Gpio {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Gpio, Command<Self::Message>) {
        (
            Self {
                config_filename: None,
                gpio_config: GPIOConfig::default(),
                pin_function_selected: [PinFunction::None; 40],
                chosen_layout: Layout::BoardLayout,
                hardware_description: None, // Until listener is ready
                listener_sender: None,      // Until listener is ready
                pin_descriptions: None,     // Until listener is ready
                pin_states: core::array::from_fn(|_| PinState::new()),
                toasts: Vec::new(),
                timeout_secs: toast::DEFAULT_TIMEOUT,
            },
            Command::batch(vec![Command::perform(
                Self::load(env::args().nth(1)),
                |result| match result {
                    Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                    _ => Message::None,
                },
            )]),
        )
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Activate(pin_number) => println!("Pin {pin_number} clicked"),
            Message::PinFunctionSelected(board_pin_number, bcm_pin_number, pin_function) => {
                self.new_pin_function(board_pin_number, bcm_pin_number, pin_function);
            }
            Message::LayoutChanged(layout) => {
                self.chosen_layout = layout;
                let layout_size = Self::get_dimensions_for_layout(layout);
                return window::resize(window::Id::MAIN, layout_size);
            }

            Message::ConfigLoaded((filename, config)) => {
                self.config_filename = Some(filename);
                self.gpio_config = config;
                self.set_pin_functions_after_load();
                self.update_hw_config();
            }
            Message::Save => {
                let gpio_config = self.gpio_config.clone();
                return Command::perform(
                    Self::save_via_picker(gpio_config),
                    |result| match result {
                        Ok(_) => Message::None,
                        _ => Message::None, // eprintln ! ("Error saving configuration to {}: {}", path_str, err);
                    },
                );
            }
            Message::Load => {
                return Command::perform(Self::load_via_picker(), |result| match result {
                    Ok(Some((filename, config))) => Message::ConfigLoaded((filename, config)),
                    _ => Message::None,
                })
            }
            Message::None => {}
            Message::HardwareListener(event) => match event {
                HWListenerEvent::Ready(config_change_sender, hw_desc, pins) => {
                    self.listener_sender = Some(config_change_sender);
                    self.hardware_description = Some(hw_desc);
                    self.pin_descriptions = Some(pins);
                    self.set_pin_functions_after_load();
                    self.update_hw_config();
                }
                HWListenerEvent::InputChange(level_change) => {
                    if let Some(pins) = &self.pin_descriptions {
                        if let Some(board_pin_number) =
                            pins.bcm_to_board(level_change.bcm_pin_number)
                        {
                            self.pin_states[board_pin_number as usize - 1].set_level(level_change);
                        }
                    }
                }
            },
            Message::ChangeOutputLevel(level_change) => {
                if let Some(pins) = &self.pin_descriptions {
                    if let Some(board_pin_number) = pins.bcm_to_board(level_change.bcm_pin_number) {
                        self.pin_states[board_pin_number as usize - 1]
                            .set_level(level_change.clone());
                    }
                    if let Some(ref mut listener) = &mut self.listener_sender {
                        let _ = listener.try_send(HardwareEvent::OutputLevelChanged(level_change));
                    }
                }
            }
            Message::Add => {
                self.toasts.clear();
                self.toasts.push(Toast {
                    title: "About Piggui".into(),
                    body: version().into(),
                    status: toast::Status::Primary,
                });
                return Command::none();
            }
            Message::Close(index) => {
                self.toasts.remove(index);
                return Command::none();
            }
            Message::Timeout(timeout) => {
                self.timeout_secs = timeout as u64;
                return Command::none();
            }
            Message::Event(_) => {
                return Command::none();
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let layout_selector = pick_list(
            &Layout::ALL[..],
            Some(self.chosen_layout),
            Message::LayoutChanged,
        )
        .width(Length::Shrink)
        .placeholder("Choose Layout");

        let mut main_row = Row::new();

        if let Some(hw_desc) = &self.hardware_description {
            let layout_row = Row::new()
                .push(layout_selector)
                .align_items(Alignment::Start)
                .spacing(10);

            let hardware_desc_row = Row::new()
                .push(hardware_view(hw_desc))
                .align_items(Alignment::Center);

            let file_button_style = CustomButton {
                bg_color: Color::new(0.0, 1.0, 1.0, 1.0),
                text_color: Color::BLACK,
                hovered_bg_color: Color::new(0.0, 0.8, 0.8, 1.0),
                hovered_text_color: Color::WHITE,
                border_radius: 2.0,
            };

            let mut configuration_column = Column::new().align_items(Alignment::Start).spacing(10);
            configuration_column = configuration_column.push(layout_row);
            configuration_column = configuration_column.push(hardware_desc_row);
            configuration_column = configuration_column.push(
                Button::new(Text::new("Save Configuration"))
                    .style(file_button_style.get_button_style())
                    .on_press(Message::Save),
            );
            configuration_column = configuration_column.push(
                Button::new(Text::new("Load Configuration"))
                    .style(file_button_style.get_button_style())
                    .on_press(Message::Load),
            );

            let version_text = Text::new(version().lines().next().unwrap_or_default().to_string());

            let add_toast_button = Button::new(version_text).on_press(Message::Add);

            let version_row = Row::new()
                .push(add_toast_button)
                .align_items(Alignment::Start);

            main_row = main_row.push(
                Column::new()
                    .push(configuration_column)
                    .push(version_row)
                    .align_items(Alignment::Start)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(self.chosen_layout.get_spacing()),
            );
        }

        if let Some(pins) = &self.pin_descriptions {
            let pin_layout = match self.chosen_layout {
                Layout::BoardLayout => board_pin_layout_view(pins, self),
                Layout::BCMLayout => bcm_pin_layout_view(pins, self),
            };

            main_row = main_row.push(
                Column::new()
                    .push(pin_layout)
                    .align_items(Alignment::Center)
                    .height(Length::Fill)
                    .width(Length::Fill),
            );
        }

        let content = container(main_row)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y()
            .padding(10);

        toast::Manager::new(content, &self.toasts, Message::Close)
            .timeout(self.timeout_secs)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        hw_listener::subscribe().map(Message::HardwareListener)
    }
}
// Hardware Configuration Display
fn hardware_view(hardware_description: &HardwareDescriptor) -> Element<'static, Message> {
    let hardware_info = Column::new()
        .push(Text::new(format!(
            "Hardware: {}",
            hardware_description.hardware
        )))
        .push(Text::new(format!(
            "Revision: {}",
            hardware_description.revision
        )))
        .push(Text::new(format!(
            "Serial: {}",
            hardware_description.serial
        )))
        .push(Text::new(format!("Model: {}", hardware_description.model)))
        .spacing(10)
        .width(Length::Shrink)
        .align_items(Alignment::Start);

    container(hardware_info)
        .padding(10)
        .width(Length::Shrink)
        .align_x(alignment::Horizontal::Center)
        .into()
}

mod toast {
    use std::fmt;
    use std::time::{Duration, Instant};

    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Operation, Tree};
    use iced::advanced::{Clipboard, Shell, Widget};
    use iced::event::{self, Event};
    use iced::mouse;
    use iced::theme;
    use iced::widget::{button, column, container, horizontal_rule, horizontal_space, row, text};
    use iced::window;
    use iced::{Alignment, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

    pub const DEFAULT_TIMEOUT: u64 = 5;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Status {
        #[default]
        Primary,
        Secondary,
        Success,
        Danger,
    }

    impl Status {
        pub const ALL: &'static [Self] =
            &[Self::Primary, Self::Secondary, Self::Success, Self::Danger];
    }

    impl container::StyleSheet for Status {
        type Style = Theme;

        fn appearance(&self, theme: &Theme) -> container::Appearance {
            let palette = theme.extended_palette();

            let pair = match self {
                Status::Primary => palette.primary.weak,
                Status::Secondary => palette.secondary.weak,
                Status::Success => palette.success.weak,
                Status::Danger => palette.danger.weak,
            };

            container::Appearance {
                background: Some(pair.color.into()),
                text_color: pair.text.into(),
                ..Default::default()
            }
        }
    }

    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Status::Primary => "Primary",
                Status::Secondary => "Secondary",
                Status::Success => "Success",
                Status::Danger => "Danger",
            }
            .fmt(f)
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct Toast {
        pub title: String,
        pub body: String,
        pub status: Status,
    }

    pub struct Manager<'a, Message> {
        content: Element<'a, Message>,
        toasts: Vec<Element<'a, Message>>,
        timeout_secs: u64,
        on_close: Box<dyn Fn(usize) -> Message + 'a>,
    }

    impl<'a, Message> Manager<'a, Message>
    where
        Message: 'a + Clone,
    {
        pub fn new(
            content: impl Into<Element<'a, Message>>,
            toasts: &'a [Toast],
            on_close: impl Fn(usize) -> Message + 'a,
        ) -> Self {
            let toasts = toasts
                .iter()
                .enumerate()
                .map(|(index, toast)| {
                    container(column![
                        container(
                            row![
                                text(toast.title.as_str()),
                                horizontal_space(),
                                button("X").on_press((on_close)(index)).padding(3),
                            ]
                            .align_items(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .padding(5)
                        .style(theme::Container::Custom(Box::new(toast.status))),
                        horizontal_rule(1),
                        container(text(toast.body.as_str()))
                            .width(Length::Fill)
                            .padding(5)
                            .style(theme::Container::Box),
                    ])
                    .width(550)
                    .into()
                })
                .collect();

            Self {
                content: content.into(),
                toasts,
                timeout_secs: DEFAULT_TIMEOUT,
                on_close: Box::new(on_close),
            }
        }

        pub fn timeout(self, seconds: u64) -> Self {
            Self {
                timeout_secs: seconds,
                ..self
            }
        }
    }

    impl<'a, Message> Widget<Message, Theme, Renderer> for Manager<'a, Message> {
        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn tag(&self) -> widget::tree::Tag {
            struct Marker;
            widget::tree::Tag::of::<Marker>()
        }

        fn state(&self) -> widget::tree::State {
            widget::tree::State::new(Vec::<Option<Instant>>::new())
        }

        fn children(&self) -> Vec<Tree> {
            std::iter::once(Tree::new(&self.content))
                .chain(self.toasts.iter().map(Tree::new))
                .collect()
        }

        fn diff(&self, tree: &mut Tree) {
            let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

            // Invalidating removed instants to None allows us to remove
            // them here so that diffing for removed / new toast instants
            // is accurate
            instants.retain(Option::is_some);

            match (instants.len(), self.toasts.len()) {
                (old, new) if old > new => {
                    instants.truncate(new);
                }
                (old, new) if old < new => {
                    instants.extend(std::iter::repeat(Some(Instant::now())).take(new - old));
                }
                _ => {}
            }

            tree.diff_children(
                &std::iter::once(&self.content)
                    .chain(self.toasts.iter())
                    .collect::<Vec<_>>(),
            );
        }

        fn operate(
            &self,
            state: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation<Message>,
        ) {
            operation.container(None, layout.bounds(), &mut |operation| {
                self.content.as_widget().operate(
                    &mut state.children[0],
                    layout,
                    renderer,
                    operation,
                );
            });
        }

        fn on_event(
            &mut self,
            state: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.content.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }

        fn draw(
            &self,
            state: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn mouse_interaction(
            &self,
            state: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
            let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

            let (content_state, toasts_state) = state.children.split_at_mut(1);

            let content = self.content.as_widget_mut().overlay(
                &mut content_state[0],
                layout,
                renderer,
                translation,
            );

            let toasts = (!self.toasts.is_empty()).then(|| {
                overlay::Element::new(Box::new(Overlay {
                    position: layout.bounds().position() + translation,
                    toasts: &mut self.toasts,
                    state: toasts_state,
                    instants,
                    on_close: &self.on_close,
                    timeout_secs: self.timeout_secs,
                }))
            });
            let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

            (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
        }
    }

    struct Overlay<'a, 'b, Message> {
        position: Point,
        toasts: &'b mut [Element<'a, Message>],
        state: &'b mut [Tree],
        instants: &'b mut [Option<Instant>],
        on_close: &'b dyn Fn(usize) -> Message,
        timeout_secs: u64,
    }

    impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, 'b, Message> {
        fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, bounds);

            layout::flex::resolve(
                layout::flex::Axis::Vertical,
                renderer,
                &limits,
                Length::Fill,
                Length::Fill,
                10.into(),
                10.0,
                Alignment::End,
                self.toasts,
                self.state,
            )
            .translate(Vector::new(self.position.x, self.position.y))
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            if let Event::Window(_, window::Event::RedrawRequested(now)) = &event {
                let mut next_redraw: Option<window::RedrawRequest> = None;

                self.instants
                    .iter_mut()
                    .enumerate()
                    .for_each(|(index, maybe_instant)| {
                        if let Some(instant) = maybe_instant.as_mut() {
                            let remaining = Duration::from_secs(self.timeout_secs)
                                .saturating_sub(instant.elapsed());

                            if remaining == Duration::ZERO {
                                maybe_instant.take();
                                shell.publish((self.on_close)(index));
                                next_redraw = Some(window::RedrawRequest::NextFrame);
                            } else {
                                let redraw_at = window::RedrawRequest::At(*now + remaining);
                                next_redraw = next_redraw
                                    .map(|redraw| redraw.min(redraw_at))
                                    .or(Some(redraw_at));
                            }
                        }
                    });

                if let Some(redraw) = next_redraw {
                    shell.request_redraw(redraw);
                }
            }

            let viewport = layout.bounds();

            self.toasts
                .iter_mut()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .zip(self.instants.iter_mut())
                .map(|(((child, state), layout), instant)| {
                    let mut local_messages = vec![];
                    let mut local_shell = Shell::new(&mut local_messages);

                    let status = child.as_widget_mut().on_event(
                        state,
                        event.clone(),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        &mut local_shell,
                        &viewport,
                    );

                    if !local_shell.is_empty() {
                        instant.take();
                    }

                    shell.merge(local_shell, std::convert::identity);

                    status
                })
                .fold(event::Status::Ignored, event::Status::merge)
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            let viewport = layout.bounds();

            for ((child, state), layout) in self
                .toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
            {
                child
                    .as_widget()
                    .draw(state, renderer, theme, style, layout, cursor, &viewport);
            }
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            operation.container(None, layout.bounds(), &mut |operation| {
                self.toasts
                    .iter()
                    .zip(self.state.iter_mut())
                    .zip(layout.children())
                    .for_each(|((child, state), layout)| {
                        child
                            .as_widget()
                            .operate(state, layout, renderer, operation);
                    });
            });
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.toasts
                .iter()
                .zip(self.state.iter())
                .zip(layout.children())
                .map(|((child, state), layout)| {
                    child
                        .as_widget()
                        .mouse_interaction(state, layout, cursor, viewport, renderer)
                })
                .max()
                .unwrap_or_default()
        }

        fn is_over(
            &self,
            layout: Layout<'_>,
            _renderer: &Renderer,
            cursor_position: Point,
        ) -> bool {
            layout
                .children()
                .any(|layout| layout.bounds().contains(cursor_position))
        }
    }

    impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
    where
        Message: 'a,
    {
        fn from(manager: Manager<'a, Message>) -> Self {
            Element::new(manager)
        }
    }
}
