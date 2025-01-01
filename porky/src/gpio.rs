use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::InputPull;
use crate::hw_definition::config::{HardwareConfig, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::{Input, Output};
use crate::hw_definition::{BCMPinNumber, PinLevel};
use crate::HARDWARE_EVENT_CHANNEL;
#[cfg(feature = "wifi")]
use cyw43::Control;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::Flex;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Pull;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::Instant;
use heapless::FnvIndexMap;

/// The configured/not-configured state of the GPIO Pins on the Pi Pico, and how to access them
/// as 3 of them are accessed via Cyw43 and others via gpio API
pub enum GPIOPin<'a> {
    Available(Flex<'a>),
    GPIOInput(
        (
            Sender<'a, ThreadModeRawMutex, bool, 1>,
            Receiver<'a, ThreadModeRawMutex, Flex<'a>, 1>,
        ),
    ),
    #[cfg(feature = "wifi")]
    CYW43Input,
    #[cfg(feature = "wifi")]
    CYW43Output,
    GPIOOutput(Flex<'a>),
}

pub static mut GPIO_PINS: FnvIndexMap<BCMPinNumber, GPIOPin, 32> = FnvIndexMap::new();

static RETURNER: Channel<ThreadModeRawMutex, Flex, 1> = Channel::new();
static SIGNALLER: Channel<ThreadModeRawMutex, bool, 1> = Channel::new();

/// Wait until a level change on an input occurs and then send it via TCP to GUI
#[embassy_executor::task(pool_size = 32)]
async fn monitor_input(
    bcm_pin_number: BCMPinNumber,
    signaller: Receiver<'static, ThreadModeRawMutex, bool, 1>,
    returner: Sender<'static, ThreadModeRawMutex, Flex<'static>, 1>,
    mut flex: Flex<'static>,
) {
    let mut level = flex.get_level();
    send_input_level(bcm_pin_number, level).await;

    loop {
        match select(flex.wait_for_any_edge(), signaller.receive()).await {
            Either::First(()) => {
                let new_level = flex.get_level();
                if new_level != level {
                    send_input_level(bcm_pin_number, flex.get_level()).await;
                    level = new_level;
                }
            }
            Either::Second(_) => {
                info!("Monitor returning Pin");
                // Return the Flex pin and exit the task
                let _ = returner.send(flex).await;
                break;
            }
        }
    }
}

/// Send a detected input level change back to the GUI, timestamping with the Duration since boot
async fn send_input_level(bcm: BCMPinNumber, level: Level) {
    let level_change = LevelChange::new(
        level == Level::High,
        Instant::now().duration_since(Instant::MIN).into(),
    );
    let hardware_event = IOLevelChanged(bcm, level_change);
    HARDWARE_EVENT_CHANNEL.sender().send(hardware_event).await;
}

fn into_level(value: PinLevel) -> Level {
    match value {
        true => Level::High,
        false => Level::Low,
    }
}

/// Set an output's level using the bcm pin number
async fn set_output_level<'a>(
    #[cfg(feature = "wifi")] control: &mut Control<'_>,
    bcm_pin_number: BCMPinNumber,
    pin_level: PinLevel,
) {
    info!(
        "Pin #{} Output level change: {:?}",
        bcm_pin_number, pin_level
    );

    // GPIO 0 and 1 are connected via cyw43 Wi-Fi chip
    unsafe {
        match GPIO_PINS.get_mut(&bcm_pin_number) {
            #[cfg(feature = "wifi")]
            Some(GPIOPin::CYW43Output) => control.gpio_set(bcm_pin_number, pin_level).await,
            Some(GPIOPin::GPIOOutput(flex)) => flex.set_level(into_level(pin_level)),
            _ => error!("Pin {} is not configured as an Output", bcm_pin_number),
        }
    }
}

/// Apply the requested config to one pin, using bcm_pin_number
async fn apply_pin_config<'a>(
    #[cfg(feature = "wifi")] control: &mut Control<'_>,
    spawner: &Spawner,
    bcm_pin_number: BCMPinNumber,
    new_pin_function: &PinFunction,
) {
    let flex_pin = unsafe {
        match GPIO_PINS.remove(&bcm_pin_number) {
            // Pin was set up as an Input - recover the Flex
            Some(GPIOPin::GPIOInput((signaller, returner))) => {
                // Signal to pin monitor to exit
                signaller.send(true).await;
                // Recover the Flex
                Some(returner.receive().await)
            }
            // Pin is available - was unassigned
            Some(GPIOPin::Available(flex)) => Some(flex),
            // Was assigned as an output - recover the Flex
            Some(GPIOPin::GPIOOutput(flex)) => Some(flex),
            // The cyw43 pins cannot be changed - just used
            #[cfg(feature = "wifi")]
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
            if let Some(flex) = flex_pin {
                unsafe {
                    let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::Available(flex));
                }
            }
            info!("Pin #{} - Set as Available", bcm_pin_number);
        }

        Input(pull) => {
            match flex_pin {
                Some(mut flex) => {
                    flex.set_as_input();
                    info!("Pin #{} Configured as GPIO input", bcm_pin_number);

                    match pull {
                        None | Some(InputPull::None) => flex.set_pull(Pull::None),
                        Some(InputPull::PullUp) => flex.set_pull(Pull::Up),
                        Some(InputPull::PullDown) => flex.set_pull(Pull::Down),
                    };

                    if let Err(e) = spawner.spawn(monitor_input(
                        bcm_pin_number,
                        SIGNALLER.receiver(),
                        RETURNER.sender(),
                        flex,
                    )) {
                        error!("Spawn Error: {}", e);
                    }

                    unsafe {
                        let _ = GPIO_PINS.insert(
                            bcm_pin_number,
                            GPIOPin::GPIOInput((SIGNALLER.sender(), RETURNER.receiver())),
                        );
                    }
                }
                None => {
                    #[cfg(feature = "wifi")]
                    {
                        // Must be GPIO 2 is connected via cyw43 Wi-Fi chip
                        unsafe {
                            let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::CYW43Input);
                        }
                        info!("Pin #{} - Configured as input via cyw43", bcm_pin_number);
                    }
                }
            }
        }

        Output(pin_level) => {
            match flex_pin {
                Some(mut flex) => {
                    if let Some(l) = pin_level {
                        flex.set_level(into_level(*l));
                        info!("Pin #{} - Output level set to '{}'", bcm_pin_number, l);
                    }

                    flex.set_as_output();
                    info!("Pin #{} Flex pin configured as Output", bcm_pin_number);

                    unsafe {
                        let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::GPIOOutput(flex));
                    }
                }
                None => {
                    #[cfg(feature = "wifi")]
                    {
                        // Must be GPIO 0 and 1 are connected via cyw43 Wi-Fi chip
                        info!("Pin #{} cyw43 pin used as Output ", bcm_pin_number);

                        if let Some(l) = pin_level {
                            control.gpio_set(bcm_pin_number, *l).await;
                            info!("Pin #{} - output level set to '{}'", bcm_pin_number, l);
                        }
                        unsafe {
                            let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::CYW43Output);
                        }
                    }
                }
            }
        }
    }
}

/// This takes the [HardwareConfig] struct and configures all the pins in it
async fn apply_config<'a>(
    #[cfg(feature = "wifi")] control: &mut Control<'_>,
    spawner: &Spawner,
    config: &HardwareConfig,
) {
    // Config only has pins that are configured
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        apply_pin_config(
            #[cfg(feature = "wifi")]
            control,
            spawner,
            *bcm_pin_number,
            pin_function,
        )
        .await;
    }
    let num_pins = config.pin_functions.len();
    if num_pins > 0 {
        info!("New config applied - {} pins reconfigured", num_pins);
    }
}

/// Apply a config change to the hardware
/// NOTE: Initially the callback to Config/PinConfig change was async, and that compiles and runs
/// but wasn't working - so this uses a sync callback again to fix that, and an async version of
/// send_input_level() for use directly from the async context
pub async fn apply_config_change<'a>(
    #[cfg(feature = "wifi")] control: &mut Control<'_>,
    spawner: &Spawner,
    config_change: &HardwareConfigMessage,
    hardware_config: &mut HardwareConfig,
) {
    match config_change {
        NewConfig(config) => {
            apply_config(
                #[cfg(feature = "wifi")]
                control,
                spawner,
                config,
            )
            .await;
            // Update the hardware config to reflect the change
            *hardware_config = config.clone();
        }
        NewPinConfig(bcm, pin_function) => {
            apply_pin_config(
                #[cfg(feature = "wifi")]
                control,
                spawner,
                *bcm,
                pin_function,
            )
            .await;
            // Update the hardware config to reflect the change
            let _ = hardware_config.pin_functions.insert(*bcm, *pin_function);
        }
        IOLevelChanged(bcm, level_change) => {
            set_output_level(
                #[cfg(feature = "wifi")]
                control,
                *bcm,
                level_change.new_level,
            )
            .await;
            // Update the hardware config to reflect the change
            let _ = hardware_config
                .pin_functions
                .insert(*bcm, Output(Some(level_change.new_level)));
        }
        HardwareConfigMessage::GetConfig => { /* Nothing to do in GPIO */ }
    }
}
