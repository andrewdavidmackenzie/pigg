mod gpio;
mod hw;

// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::{button, container, row, Column, Text};
use iced::{alignment, window, Element, Length, Sandbox, Settings};
use crate::gpio::{GPIO_DESCRIPTION, PinDescription};
// Use Hardware via trait
//use hw::Hardware;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
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
    gpio_config: [PinDescription; 40],
    clicked: bool,
}

#[derive(Debug, Clone, Copy)]

enum Message {
    Activate,
}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        Self {
            gpio_config: GPIO_DESCRIPTION,
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
        container(pin_view(&self.gpio_config))
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.75
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn pin_view(pin_descriptions: &[PinDescription; 40]) -> Element<'static, Message> {
    let mut column = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    for pair in pin_descriptions.chunks(2) {
        let row = row!(
            Text::new(pair[0].name).size(20),
            button(Text::new(pair[0].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed( 50f32 )),
            button(Text::new(pair[1].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed( 50f32 )),
            Text::new(pair[1].name).size(20),
        )
        .spacing(10)
        .align_items(iced::Alignment::Center);
        column = column.push(row);
    }
    container(column).height(2000).width(2000).into()
}
