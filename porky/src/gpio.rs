use crate::gpio::GPIOPin::Available;
use crate::gpio_input_monitor::monitor_input;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::InputPull;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::{Input, Output};
use crate::hw_definition::{BCMPinNumber, PinLevel};
#[cfg(feature = "wifi")]
use cyw43::Control;
use defmt::{debug, error, info};
use embassy_executor::Spawner;
use embassy_rp::gpio::Flex;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::{
    PIN_0, PIN_1, PIN_10, PIN_11, PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19,
    PIN_2, PIN_20, PIN_21, PIN_22, PIN_26, PIN_27, PIN_28, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7,
    PIN_8, PIN_9,
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::{Receiver, Sender};
use heapless::FnvIndexMap;
use static_cell::StaticCell;

/// The configured/not-configured state of the GPIO Pins on the Pi Pico, and how to access them
/// as 3 of them are accessed via Cyw43 and others via gpio API
enum GPIOPin<'a> {
    Available(Flex<'a>),
    GPIOInput(
        (
            Sender<'static, ThreadModeRawMutex, bool, 1>,
            Receiver<'static, ThreadModeRawMutex, Flex<'static>, 1>,
        ),
    ),
    #[cfg(feature = "wifi")]
    CYW43Input,
    #[cfg(feature = "wifi")]
    CYW43Output,
    GPIOOutput(Flex<'a>),
}

fn into_level(value: PinLevel) -> Level {
    match value {
        true => Level::High,
        false => Level::Low,
    }
}

pub struct Gpio {
    pins: FnvIndexMap<BCMPinNumber, GPIOPin<'static>, 32>,
    returner_receiver: Receiver<'static, ThreadModeRawMutex, Flex<'static>, 1>,
    signaller_sender: Sender<'static, ThreadModeRawMutex, bool, 1>,
    signaller_receiver: Receiver<'static, ThreadModeRawMutex, bool, 1>,
    returner_sender: Sender<'static, ThreadModeRawMutex, Flex<'static>, 1>,
}

impl Gpio {
    /// Create new Gpio controller with pins to be used
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
    /// Take the set of available pins not used by other functions, including the three pins that
    /// are connected via the CYW43 Wi-Fi chip. Create [Flex] Pins out of each of the GPIO pins.
    /// Put them all into the GPIO_PINS map, marking them as available
    /// NOTE: All pin numbers are GPIO (BCM) Pin Numbers, not physical pin numbers
    /// Take the following pins out of peripherals for use a GPIO
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pin_0: PIN_0,
        pin_1: PIN_1,
        pin_2: PIN_2,
        pin_3: PIN_3,
        pin_4: PIN_4,
        pin_5: PIN_5,
        pin_6: PIN_6,
        pin_7: PIN_7,
        pin_8: PIN_8,
        pin_9: PIN_9,
        pin_10: PIN_10,
        pin_11: PIN_11,
        pin_12: PIN_12,
        pin_13: PIN_13,
        pin_14: PIN_14,
        pin_15: PIN_15,
        pin_16: PIN_16,
        pin_17: PIN_17,
        pin_18: PIN_18,
        pin_19: PIN_19,
        pin_20: PIN_20,
        pin_21: PIN_21,
        pin_22: PIN_22,
        pin_26: PIN_26,
        pin_27: PIN_27,
        pin_28: PIN_28,
    ) -> Self {
        let mut pins = FnvIndexMap::new();

        #[cfg(not(feature = "debug-probe"))]
        let _ = pins.insert(0, Available(Flex::new(pin_0)));
        #[cfg(not(feature = "debug-probe"))]
        let _ = pins.insert(1, Available(Flex::new(pin_1)));
        let _ = pins.insert(2, Available(Flex::new(pin_2)));
        let _ = pins.insert(3, Available(Flex::new(pin_3)));
        let _ = pins.insert(4, Available(Flex::new(pin_4)));
        let _ = pins.insert(5, Available(Flex::new(pin_5)));
        let _ = pins.insert(6, Available(Flex::new(pin_6)));
        let _ = pins.insert(7, Available(Flex::new(pin_7)));
        let _ = pins.insert(8, Available(Flex::new(pin_8)));
        let _ = pins.insert(9, Available(Flex::new(pin_9)));
        let _ = pins.insert(10, Available(Flex::new(pin_10)));
        let _ = pins.insert(11, Available(Flex::new(pin_11)));
        let _ = pins.insert(12, Available(Flex::new(pin_12)));
        let _ = pins.insert(13, Available(Flex::new(pin_13)));
        let _ = pins.insert(14, Available(Flex::new(pin_14)));
        let _ = pins.insert(15, Available(Flex::new(pin_15)));
        let _ = pins.insert(16, Available(Flex::new(pin_16)));
        let _ = pins.insert(17, Available(Flex::new(pin_17)));
        let _ = pins.insert(18, Available(Flex::new(pin_18)));
        let _ = pins.insert(19, Available(Flex::new(pin_19)));
        let _ = pins.insert(20, Available(Flex::new(pin_20)));
        let _ = pins.insert(21, Available(Flex::new(pin_21)));
        let _ = pins.insert(22, Available(Flex::new(pin_22)));
        let _ = pins.insert(26, Available(Flex::new(pin_26)));
        let _ = pins.insert(27, Available(Flex::new(pin_27)));
        let _ = pins.insert(28, Available(Flex::new(pin_28)));

        static RETURNER: StaticCell<Channel<ThreadModeRawMutex, Flex<'static>, 1>> =
            StaticCell::new();
        let returner = RETURNER.init(Channel::new());

        static SIGNALLER: StaticCell<Channel<ThreadModeRawMutex, bool, 1>> = StaticCell::new();
        let signaller = SIGNALLER.init(Channel::new());

        Gpio {
            pins,
            returner_receiver: returner.receiver(),
            signaller_sender: signaller.sender(),
            signaller_receiver: signaller.receiver(),
            returner_sender: returner.sender(),
        }
    }

    /// Set an output's level using the bcm pin number
    async fn set_output_level(
        &mut self,
        #[cfg(feature = "wifi")] control: &mut Control<'_>,
        bcm_pin_number: BCMPinNumber,
        pin_level: PinLevel,
    ) {
        info!(
            "Pin #{} Output level change: {:?}",
            bcm_pin_number, pin_level
        );

        // GPIO 0 and 1 are connected via cyw43 Wi-Fi chip
        match self.pins.get_mut(&bcm_pin_number) {
            #[cfg(feature = "wifi")]
            Some(GPIOPin::CYW43Output) => control.gpio_set(bcm_pin_number, pin_level).await,
            Some(GPIOPin::GPIOOutput(flex)) => flex.set_level(into_level(pin_level)),
            _ => error!("Pin {} is not configured as an Output", bcm_pin_number),
        }
    }

    /// Apply the requested config to one pin, using bcm_pin_number
    async fn apply_pin_config(
        &mut self,
        #[cfg(feature = "wifi")] control: &mut Control<'_>,
        spawner: &Spawner,
        bcm_pin_number: BCMPinNumber,
        new_pin_function: &PinFunction,
    ) {
        let flex_pin = {
            match self.pins.remove(&bcm_pin_number) {
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
                    let _ = self.pins.insert(bcm_pin_number, GPIOPin::Available(flex));
                }
                debug!("Pin #{} - Set as Available", bcm_pin_number);
            }

            Input(pull) => {
                match flex_pin {
                    Some(mut flex) => {
                        flex.set_as_input();
                        debug!("Pin #{} Configured as GPIO input", bcm_pin_number);

                        match pull {
                            None | Some(InputPull::None) => flex.set_pull(Pull::None),
                            Some(InputPull::PullUp) => flex.set_pull(Pull::Up),
                            Some(InputPull::PullDown) => flex.set_pull(Pull::Down),
                        };

                        if let Err(e) = spawner.spawn(monitor_input(
                            bcm_pin_number,
                            self.signaller_receiver,
                            self.returner_sender,
                            flex,
                        )) {
                            error!("Spawn Error: {}", e);
                        }

                        let _ = self.pins.insert(
                            bcm_pin_number,
                            GPIOPin::GPIOInput((self.signaller_sender, self.returner_receiver)),
                        );
                    }
                    None => {
                        #[cfg(feature = "wifi")]
                        {
                            // Must be GPIO 2 is connected via cyw43 Wi-Fi chip
                            let _ = self.pins.insert(bcm_pin_number, GPIOPin::CYW43Input);
                            debug!("Pin #{} - Configured as input via cyw43", bcm_pin_number);
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

                        let _ = self.pins.insert(bcm_pin_number, GPIOPin::GPIOOutput(flex));
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
                            let _ = self.pins.insert(bcm_pin_number, GPIOPin::CYW43Output);
                        }
                    }
                }
            }
        }
    }

    /// This takes the [HardwareConfig] struct and configures all the pins in it
    async fn apply_config(
        &mut self,
        #[cfg(feature = "wifi")] control: &mut Control<'_>,
        spawner: &Spawner,
        config: &HardwareConfig,
    ) {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.pin_functions {
            self.apply_pin_config(
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
    pub async fn apply_config_change(
        &mut self,
        #[cfg(feature = "wifi")] control: &mut Control<'_>,
        spawner: &Spawner,
        config_change: &HardwareConfigMessage,
        hardware_config: &mut HardwareConfig,
    ) {
        match config_change {
            NewConfig(config) => {
                self.apply_config(
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
                self.apply_pin_config(
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
                self.set_output_level(
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
            HardwareConfigMessage::Disconnect => { /* Nothing to do in GPIO */ }
        }
    }
}
