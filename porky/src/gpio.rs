use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::*;
use crate::hw_definition::config::InputPull;
use crate::hw_definition::config::{HardwareConfig, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};
use cyw43::Control;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::Flex;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Pull;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::Instant;
use embedded_io_async::Write;
use heapless::FnvIndexMap;

/// The configured/not-configured state of the GPIO Pins on the Pi Pico, and how to access them
/// as 3 of them are accessed via Cyw43 and others via gpio API
enum GPIOPin<'a> {
    Available(Flex<'a>),
    GPIOInput(
        (
            Sender<'a, ThreadModeRawMutex, bool, 1>,
            Receiver<'a, ThreadModeRawMutex, Flex<'a>, 1>,
        ),
    ),
    CYW43Input,
    CYW43Output,
    GPIOOutput(Flex<'a>),
}

pub struct AvailablePins {
    pub pin_3: embassy_rp::peripherals::PIN_3,
    pub pin_4: embassy_rp::peripherals::PIN_4,
    pub pin_5: embassy_rp::peripherals::PIN_5,
    pub pin_6: embassy_rp::peripherals::PIN_6,
    pub pin_7: embassy_rp::peripherals::PIN_7,
    pub pin_8: embassy_rp::peripherals::PIN_8,
    pub pin_9: embassy_rp::peripherals::PIN_9,
    pub pin_10: embassy_rp::peripherals::PIN_10,
    pub pin_11: embassy_rp::peripherals::PIN_11,
    pub pin_12: embassy_rp::peripherals::PIN_12,
    pub pin_13: embassy_rp::peripherals::PIN_13,
    pub pin_14: embassy_rp::peripherals::PIN_14,
    pub pin_15: embassy_rp::peripherals::PIN_15,
    pub pin_16: embassy_rp::peripherals::PIN_16,
    pub pin_17: embassy_rp::peripherals::PIN_17,
    pub pin_18: embassy_rp::peripherals::PIN_18,
    pub pin_19: embassy_rp::peripherals::PIN_19,
    pub pin_20: embassy_rp::peripherals::PIN_20,
    pub pin_21: embassy_rp::peripherals::PIN_21,
    pub pin_22: embassy_rp::peripherals::PIN_22,
    pub pin_26: embassy_rp::peripherals::PIN_26,
    pub pin_27: embassy_rp::peripherals::PIN_27,
    pub pin_28: embassy_rp::peripherals::PIN_28,
}

static mut GPIO_PINS: FnvIndexMap<BCMPinNumber, GPIOPin, 32> = FnvIndexMap::new();

static RETURNER: Channel<ThreadModeRawMutex, Flex, 1> = Channel::new();
static SIGNALLER: Channel<ThreadModeRawMutex, bool, 1> = Channel::new();

/// Wait until a level change on an input occurs and then send it via TCP to GUI
#[embassy_executor::task]
pub async fn monitor_input(
    _bcm_pin_number: BCMPinNumber,
    //    socket: &mut TcpSocket<'_>,
    signaller: Receiver<'static, ThreadModeRawMutex, bool, 1>,
    returner: Sender<'static, ThreadModeRawMutex, Flex<'static>, 1>,
    mut flex: Flex<'static>,
) {
    //    let _ = send_input_level(socket, bcm_pin_number, flex.get_level()).await;

    loop {
        match select(flex.wait_for_any_edge(), signaller.receive()).await {
            Either::First(()) => {
                info!("Level change detected");
                // send_input_level(socket, bcm_pin_number, flex.get_level()).await
            }
            Either::Second(_) => {
                info!("Monitor returning Pin");
                // Return the Flex pin and exit the task
                let _ = returner.send(flex);
                break;
            }
        }
    }
}

fn into_level(value: PinLevel) -> Level {
    match value {
        true => Level::High,
        false => Level::Low,
    }
}

/// Set an output's level using the bcm pin number
async fn set_output_level<'a>(
    control: &mut Control<'_>,
    bcm_pin_number: BCMPinNumber,
    pin_level: PinLevel,
) {
    info!(
        "Pin #{} Output level change: {:?}",
        bcm_pin_number, pin_level
    );

    // GPIO 0 and 1 are connected via cyw43 wifi chip
    unsafe {
        match GPIO_PINS.get_mut(&bcm_pin_number) {
            Some(GPIOPin::CYW43Output) => control.gpio_set(bcm_pin_number, pin_level).await,
            Some(GPIOPin::GPIOOutput(flex)) => flex.set_level(into_level(pin_level)),
            _ => error!("Pin {} is not configured as an Output", bcm_pin_number),
        }
    }
}

/// Apply the requested config to one pin, using bcm_pin_number
async fn apply_pin_config<'a>(
    control: &mut Control<'_>,
    spawner: &Spawner,
    bcm_pin_number: BCMPinNumber,
    new_pin_function: &PinFunction,
    _socket: &mut TcpSocket<'_>,
) {
    let gpio_pin = unsafe {
        match GPIO_PINS.remove(&bcm_pin_number) {
            // Pin was setup as an Input - recover it for use
            Some(GPIOPin::GPIOInput((signaller, returner))) => {
                // Signal to pin monitor to exit
                signaller.send(true).await;
                // Recover the Flex
                Some(returner.receive().await)
            }
            // Pin is available - was unassigned
            Some(GPIOPin::Available(flex)) => Some(flex),
            // Was assigned as an output - so recover it
            Some(GPIOPin::GPIOOutput(flex)) => Some(flex),
            // The cyw43 pins cannot be changed - just used
            Some(GPIOPin::CYW43Input) | Some(GPIOPin::CYW43Output) => None,
            _ => {
                error!("Could not find pin #{}", bcm_pin_number);
                return;
            }
        }
    };

    match new_pin_function {
        PinFunction::None => {
            // if we recovered a pin above - then leave it as
            if let Some(flex) = gpio_pin {
                unsafe {
                    let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::Available(flex));
                }
            }
            info!("Pin #{} - Unconfigured", bcm_pin_number);
        }

        PinFunction::Input(pull) => {
            match gpio_pin {
                Some(mut flex) => {
                    flex.set_as_input();
                    info!("Pin #{} Configured as GPIO input", bcm_pin_number);

                    match pull {
                        None | Some(InputPull::None) => flex.set_pull(Pull::None),
                        Some(InputPull::PullUp) => flex.set_pull(Pull::Up),
                        Some(InputPull::PullDown) => flex.set_pull(Pull::Down),
                    };

                    spawner
                        .spawn(monitor_input(
                            bcm_pin_number,
                            //                            socket,
                            SIGNALLER.receiver(),
                            RETURNER.sender(),
                            flex,
                        ))
                        .unwrap();

                    unsafe {
                        let _ = GPIO_PINS.insert(
                            bcm_pin_number,
                            GPIOPin::GPIOInput((SIGNALLER.sender(), RETURNER.receiver())),
                        );
                    }
                }
                None => {
                    // Must be GPIO 2 is connected via cyw43 wifi chip
                    unsafe {
                        let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::CYW43Input);
                    }
                    info!("Pin #{} - Configured as input via cyw43", bcm_pin_number);
                }
            }
        }

        PinFunction::Output(pin_level) => {
            match gpio_pin {
                Some(mut flex) => {
                    flex.set_as_output();
                    info!("Pin #{} Configured as GPIO output", bcm_pin_number);

                    if let Some(l) = pin_level {
                        flex.set_level(into_level(*l));
                        info!("Pin #{} - output level set to '{}'", bcm_pin_number, l);
                    }

                    unsafe {
                        let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::GPIOOutput(flex));
                    }
                }
                None => {
                    // Must be GPIO 0 and 1 are connected via cyw43 wifi chip
                    info!("Pin #{} Configured as output via cyw43", bcm_pin_number);

                    if let Some(l) = pin_level {
                        control.gpio_set(bcm_pin_number, *l).await;
                        info!(
                            "Pin #{} - output level set to '{}'",
                            bcm_pin_number, pin_level
                        );
                    }
                    unsafe {
                        let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::CYW43Output);
                    }
                }
            }
        }
    }
}

/// This takes the GPIOConfig struct and configures all the pins in it
async fn apply_config<'a>(
    control: &mut Control<'_>,
    spawner: &Spawner,
    config: &HardwareConfig,
    socket: &mut TcpSocket<'_>,
) {
    // Config only has pins that are configured
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        apply_pin_config(control, spawner, *bcm_pin_number, pin_function, socket).await;
    }
    info!("New config applied");
}

#[allow(dead_code)] // TODO remove when finish sending input levels
/// Send a detected input level change back to the GUI using `writer` [TcpStream],
/// timestamping with the current time in Utc
async fn send_input_level(socket: &mut TcpSocket<'_>, bcm: BCMPinNumber, level: Level) {
    let level_change = LevelChange::new(
        level == Level::High,
        Instant::now().duration_since(Instant::MIN).into(),
    );
    let hardware_event = IOLevelChanged(bcm, level_change);
    let mut buf = [0; 1024];
    let message = postcard::to_slice(&hardware_event, &mut buf).unwrap();
    socket.write_all(&message).await.unwrap();
}

/// Apply a config change to the hardware
/// NOTE: Initially the callback to Config/PinConfig change was async, and that compiles and runs
/// but wasn't working - so this uses a sync callback again to fix that, and an async version of
/// send_input_level() for use directly from the async context
pub async fn apply_config_change<'a>(
    control: &mut Control<'_>,
    spawner: &Spawner,
    config_change: HardwareConfigMessage,
    socket: &mut TcpSocket<'_>,
) {
    match config_change {
        NewConfig(config) => apply_config(control, spawner, &config, socket).await,
        NewPinConfig(bcm, pin_function) => {
            apply_pin_config(control, spawner, bcm, &pin_function, socket).await
        }
        IOLevelChanged(bcm, level_change) => {
            set_output_level(control, bcm, level_change.new_level).await;
        }
    }
}

/// Take the set of available pins not used by other functions, including the three pins that
/// are connected via the CYW43 Wi-Fi chip. Create [Flex] Pins out of each of the GPIO pins.
/// Put them all into the GPIO_PINS map, marking them as available
pub fn setup_pins<'a>(available_pins: AvailablePins) {
    unsafe {
        let _ = GPIO_PINS.insert(0, GPIOPin::CYW43Output); // GP0 connected to CYW43 chip
        let _ = GPIO_PINS.insert(1, GPIOPin::CYW43Output); // GP1 connected to CYW43 chip
        let _ = GPIO_PINS.insert(2, GPIOPin::CYW43Input); // GP2 connected to CYW43 chip
        let _ = GPIO_PINS.insert(3, GPIOPin::Available(Flex::new(available_pins.pin_3)));
        let _ = GPIO_PINS.insert(4, GPIOPin::Available(Flex::new(available_pins.pin_4)));
        let _ = GPIO_PINS.insert(5, GPIOPin::Available(Flex::new(available_pins.pin_5)));
        let _ = GPIO_PINS.insert(6, GPIOPin::Available(Flex::new(available_pins.pin_6)));
        let _ = GPIO_PINS.insert(7, GPIOPin::Available(Flex::new(available_pins.pin_7)));
        let _ = GPIO_PINS.insert(8, GPIOPin::Available(Flex::new(available_pins.pin_8)));
        let _ = GPIO_PINS.insert(9, GPIOPin::Available(Flex::new(available_pins.pin_9)));
        let _ = GPIO_PINS.insert(10, GPIOPin::Available(Flex::new(available_pins.pin_10)));
        let _ = GPIO_PINS.insert(11, GPIOPin::Available(Flex::new(available_pins.pin_11)));
        let _ = GPIO_PINS.insert(12, GPIOPin::Available(Flex::new(available_pins.pin_12)));
        let _ = GPIO_PINS.insert(13, GPIOPin::Available(Flex::new(available_pins.pin_13)));
        let _ = GPIO_PINS.insert(14, GPIOPin::Available(Flex::new(available_pins.pin_14)));
        let _ = GPIO_PINS.insert(15, GPIOPin::Available(Flex::new(available_pins.pin_15)));
        let _ = GPIO_PINS.insert(16, GPIOPin::Available(Flex::new(available_pins.pin_16)));
        let _ = GPIO_PINS.insert(17, GPIOPin::Available(Flex::new(available_pins.pin_17)));
        let _ = GPIO_PINS.insert(18, GPIOPin::Available(Flex::new(available_pins.pin_18)));
        let _ = GPIO_PINS.insert(19, GPIOPin::Available(Flex::new(available_pins.pin_19)));
        let _ = GPIO_PINS.insert(20, GPIOPin::Available(Flex::new(available_pins.pin_20)));
        let _ = GPIO_PINS.insert(21, GPIOPin::Available(Flex::new(available_pins.pin_21)));
        let _ = GPIO_PINS.insert(22, GPIOPin::Available(Flex::new(available_pins.pin_22)));
        let _ = GPIO_PINS.insert(26, GPIOPin::Available(Flex::new(available_pins.pin_26)));
        let _ = GPIO_PINS.insert(27, GPIOPin::Available(Flex::new(available_pins.pin_27)));
        let _ = GPIO_PINS.insert(28, GPIOPin::Available(Flex::new(available_pins.pin_28)));
    }
}
