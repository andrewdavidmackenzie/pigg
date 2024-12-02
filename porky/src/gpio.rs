use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::InputPull;
use crate::hw_definition::config::{HardwareConfig, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::Output;
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
enum GPIOPin<'a> {
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

static mut GPIO_PINS: FnvIndexMap<BCMPinNumber, GPIOPin, 32> = FnvIndexMap::new();

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
    send_input_level(bcm_pin_number, flex.get_level()).await;

    loop {
        match select(flex.wait_for_any_edge(), signaller.receive()).await {
            Either::First(()) => send_input_level(bcm_pin_number, flex.get_level()).await,
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

    // GPIO 0 and 1 are connected via cyw43 wifi chip
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
            info!("Setting new pin function to: None");
            // if we recovered a pin above - then leave it as
            if let Some(flex) = flex_pin {
                unsafe {
                    let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::Available(flex));
                }
            }
            info!("Pin #{} - Set as Available", bcm_pin_number);
        }

        PinFunction::Input(pull) => {
            info!("Setting new pin function to: Input");
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
                        // Must be GPIO 2 is connected via cyw43 wifi chip
                        unsafe {
                            let _ = GPIO_PINS.insert(bcm_pin_number, GPIOPin::CYW43Input);
                        }
                        info!("Pin #{} - Configured as input via cyw43", bcm_pin_number);
                    }
                }
            }
        }

        PinFunction::Output(pin_level) => {
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
                        // Must be GPIO 0 and 1 are connected via cyw43 wifi chip
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
                &config,
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
                &pin_function,
            )
            .await;
            // Update the hardware config to reflect the change
            let _ = hardware_config
                .pin_functions
                .insert(*bcm, pin_function.clone());
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
    }
}

/// Pins available to be programmed, but are not connected to header
///  WL_GPIO0 - via CYW43 - Connected to user LED
///  WL_GPIO1 - via CYW43 - Output controls on-board SMPS power save pin
///  WL_GPIO2 - via CYW43 - Input VBUS sense - high if VBUS is present, else low
///
/// GPIO Pins Used Internally that are not in the list below
///  GP23 - Output wireless on signal - used by embassy
///  GP24 - Output/Input wireless SPI data/IRQ
///  GP25 - Output wireless SPI CS- when high enables GPIO29 ADC pin to read VSYS
///  GP29 - Output/Input SPI CLK/ADC Mode to measure VSYS/3
///
/// Refer to $ProjectRoot/assets/images/pi_pico_w_pinout.png
pub struct HeaderPins {
    // Physical Pin # 1 - GP0
    // Maybe in use by Debug-Probe
    #[cfg(not(feature = "debug-probe"))]
    pub pin_0: embassy_rp::peripherals::PIN_0,
    // Physical Pin # 2 - GP1
    // Maybe in use by Debug-Probe
    #[cfg(not(feature = "debug-probe"))]
    pub pin_1: embassy_rp::peripherals::PIN_1,
    // Physical Pin # 3 - GROUND
    // Physical Pin # 4 - GP2
    pub pin_2: embassy_rp::peripherals::PIN_2,
    // Physical Pin # 5 - GP3
    pub pin_3: embassy_rp::peripherals::PIN_3,
    // Physical Pin # 6 - GP4
    pub pin_4: embassy_rp::peripherals::PIN_4,
    // Physical Pin # 7 - GP5
    pub pin_5: embassy_rp::peripherals::PIN_5,
    // Physical Pin # 8 - GROUND
    // Physical Pin # 9 - GP6
    pub pin_6: embassy_rp::peripherals::PIN_6,
    // Physical Pin # 10 - GP7
    pub pin_7: embassy_rp::peripherals::PIN_7,
    // Physical Pin # 11 - GP8
    pub pin_8: embassy_rp::peripherals::PIN_8,
    // Physical Pin # 12 - GP9
    pub pin_9: embassy_rp::peripherals::PIN_9,
    // Physical Pin # 13 - GROUND
    // Physical Pin # 14 - GP10
    pub pin_10: embassy_rp::peripherals::PIN_10,
    // Physical Pin # 15 - GP11
    pub pin_11: embassy_rp::peripherals::PIN_11,
    // Physical Pin # 16 - GP12
    pub pin_12: embassy_rp::peripherals::PIN_12,
    // Physical Pin # 17 - GP13
    pub pin_13: embassy_rp::peripherals::PIN_13,
    // Physical Pin # 18 - GROUND
    // Physical Pin # 19 - GP14
    pub pin_14: embassy_rp::peripherals::PIN_14,
    // Physical Pin # 20 - GP15
    pub pin_15: embassy_rp::peripherals::PIN_15,
    // Physical Pin # 21 - GP16
    pub pin_16: embassy_rp::peripherals::PIN_16,
    // Physical Pin # 22 - GP17
    pub pin_17: embassy_rp::peripherals::PIN_17,
    // Physical Pin # 23 - GROUND
    // Physical Pin # 24 - GP18
    pub pin_18: embassy_rp::peripherals::PIN_18,
    // Physical Pin # 25 - GP19
    pub pin_19: embassy_rp::peripherals::PIN_19,
    // Physical Pin # 26 - GP20
    pub pin_20: embassy_rp::peripherals::PIN_20,
    // Physical Pin # 27 - GP21
    pub pin_21: embassy_rp::peripherals::PIN_21,
    // Physical Pin # 28 - GROUND
    // Physical Pin # 29 - GP22
    pub pin_22: embassy_rp::peripherals::PIN_22,
    // Physical Pin # 30 - RUN
    // Physical Pin # 31 - GP26
    pub pin_26: embassy_rp::peripherals::PIN_26,
    // Physical Pin # 32 - GP27
    pub pin_27: embassy_rp::peripherals::PIN_27,
    // Physical Pin # 33 - GROUND
    // Physical Pin # 34 - GP28
    pub pin_28: embassy_rp::peripherals::PIN_28,
    // Physical Pin # 35 - ADC_VREF
    // Physical Pin # 36 - 3V3
    // Physical Pin # 37 - 3V3_EN
    // Physical Pin # 38 - GROUND
    // Physical Pin # 39 - VSYS
    // Physical Pin # 40 - VBUS
}

/// Take the set of available pins not used by other functions, including the three pins that
/// are connected via the CYW43 Wi-Fi chip. Create [Flex] Pins out of each of the GPIO pins.
/// Put them all into the GPIO_PINS map, marking them as available
/// NOTE: All pin numbers are GPIO (BCM) Pin Numbers, not physical pin numbers
/// TODO we need to find a way to control these other pins
///         let _ = GPIO_PINS.insert(0, GPIOPin::CYW43Output); // GP0 connected to CYW43 chip
//         #[cfg(not(feature = "debug-probe"))]
//         let _ = GPIO_PINS.insert(1, GPIOPin::CYW43Output); // GP1 connected to CYW43 chip
//         #[cfg(not(feature = "debug-probe"))]
//         let _ = GPIO_PINS.insert(2, GPIOPin::CYW43Input); // GP2 connected to CYW43 chip
pub fn setup_pins(header_pins: HeaderPins) {
    unsafe {
        #[cfg(not(feature = "debug-probe"))]
        let _ = GPIO_PINS.insert(0, GPIOPin::Available(Flex::new(header_pins.pin_0)));
        #[cfg(not(feature = "debug-probe"))]
        let _ = GPIO_PINS.insert(1, GPIOPin::Available(Flex::new(header_pins.pin_1)));
        let _ = GPIO_PINS.insert(2, GPIOPin::Available(Flex::new(header_pins.pin_2)));
        let _ = GPIO_PINS.insert(3, GPIOPin::Available(Flex::new(header_pins.pin_3)));
        let _ = GPIO_PINS.insert(4, GPIOPin::Available(Flex::new(header_pins.pin_4)));
        let _ = GPIO_PINS.insert(5, GPIOPin::Available(Flex::new(header_pins.pin_5)));
        let _ = GPIO_PINS.insert(6, GPIOPin::Available(Flex::new(header_pins.pin_6)));
        let _ = GPIO_PINS.insert(7, GPIOPin::Available(Flex::new(header_pins.pin_7)));
        let _ = GPIO_PINS.insert(8, GPIOPin::Available(Flex::new(header_pins.pin_8)));
        let _ = GPIO_PINS.insert(9, GPIOPin::Available(Flex::new(header_pins.pin_9)));
        let _ = GPIO_PINS.insert(10, GPIOPin::Available(Flex::new(header_pins.pin_10)));
        let _ = GPIO_PINS.insert(11, GPIOPin::Available(Flex::new(header_pins.pin_11)));
        let _ = GPIO_PINS.insert(12, GPIOPin::Available(Flex::new(header_pins.pin_12)));
        let _ = GPIO_PINS.insert(13, GPIOPin::Available(Flex::new(header_pins.pin_13)));
        let _ = GPIO_PINS.insert(14, GPIOPin::Available(Flex::new(header_pins.pin_14)));
        let _ = GPIO_PINS.insert(15, GPIOPin::Available(Flex::new(header_pins.pin_15)));
        let _ = GPIO_PINS.insert(16, GPIOPin::Available(Flex::new(header_pins.pin_16)));
        let _ = GPIO_PINS.insert(17, GPIOPin::Available(Flex::new(header_pins.pin_17)));
        let _ = GPIO_PINS.insert(18, GPIOPin::Available(Flex::new(header_pins.pin_18)));
        let _ = GPIO_PINS.insert(19, GPIOPin::Available(Flex::new(header_pins.pin_19)));
        let _ = GPIO_PINS.insert(20, GPIOPin::Available(Flex::new(header_pins.pin_20)));
        let _ = GPIO_PINS.insert(21, GPIOPin::Available(Flex::new(header_pins.pin_21)));
        let _ = GPIO_PINS.insert(22, GPIOPin::Available(Flex::new(header_pins.pin_22)));
        let _ = GPIO_PINS.insert(26, GPIOPin::Available(Flex::new(header_pins.pin_26)));
        let _ = GPIO_PINS.insert(27, GPIOPin::Available(Flex::new(header_pins.pin_27)));
        let _ = GPIO_PINS.insert(28, GPIOPin::Available(Flex::new(header_pins.pin_28)));
    }
    info!("GPIO Pins setup");
}
