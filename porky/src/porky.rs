#![no_std]
#![no_main]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use crate::flash::DbFlash;
use crate::gpio::{GPIOPin, GPIO_PINS};
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};
use crate::pin_descriptions::PIN_DESCRIPTIONS;
use core::str;
use ekv::Database;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::gpio::Flex;
use embassy_rp::peripherals::FLASH;
#[cfg(feature = "usb")]
use embassy_rp::peripherals::USB;
#[cfg(feature = "usb")]
use embassy_rp::usb::Driver;
#[cfg(feature = "usb")]
use embassy_rp::usb::InterruptHandler as USBInterruptHandler;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use heapless::Vec;
use static_cell::StaticCell;
use GPIOPin::Available;

#[cfg(not(any(feature = "pico", feature = "pico_w")))]
compile_error!(
    "You must chose a feature from:[\"pico\", \"pico_w\"] to select a device to build for"
);

#[cfg(all(feature = "pico", feature = "pico_w"))]
compile_error!(
    "You must chose a just one of [\"pico\", \"pico_w\"] to select a device to build for"
);

//#[cfg(feature = "usb")]
mod usb;

/// GPIO control related functions
mod gpio;
mod gpio_input_monitor;

/// Definition of hardware structs passed back and fore between porky and the GUI
#[path = "../../src/hw_definition/mod.rs"]
mod hw_definition;

/// Functions for interacting with the Flash ROM
mod flash;

/// Persistence layer built on top of flash
mod persistence;

/// The Pi Pico GPIO [PinDefinition]s that get passed to the GUI
mod pin_descriptions;

bind_interrupts!(struct Irqs {
    #[cfg(feature = "usb")]
    USBCTRL_IRQ => USBInterruptHandler<USB>;
});

pub static HARDWARE_EVENT_CHANNEL: Channel<ThreadModeRawMutex, HardwareConfigMessage, 1> =
    Channel::new();

/// Create a [HardwareDescription] for this device with the provided serial number
fn hardware_description(serial: &str) -> HardwareDescription {
    let details = HardwareDetails {
        model: "Pi Pico",
        hardware: "RP2040",
        revision: "",
        serial,
        wifi: false,
        app_name: env!("CARGO_BIN_NAME"),
        app_version: env!("CARGO_PKG_VERSION"),
    };

    HardwareDescription {
        details,
        pins: PinDescriptionSet {
            pins: Vec::from_slice(&PIN_DESCRIPTIONS).unwrap(),
        },
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Get the RPi Pico Peripherals - a number of the PINS are available for GPIO (they are
    // passed to AvailablePins) while others are reserved for internal use and not available for
    // GPIO
    let peripherals = embassy_rp::init(Default::default());

    // Pins available to be programmed, but are not connected to header
    //  WL_GPIO0 - via CYW43 - Connected to user LED
    //  WL_GPIO1 - via CYW43 - Output controls on-board SMPS power save pin
    //  WL_GPIO2 - via CYW43 - Input VBUS sense - high if VBUS is present, else low
    //
    // GPIO Pins Used Internally that are not in the list below
    //  GP23 - Output wireless on signal - used by embassy
    //  GP24 - Output/Input wireless SPI data/IRQ
    //  GP25 - Output wireless SPI CS- when high enables GPIO29 ADC pin to read VSYS
    //  GP29 - Output/Input SPI CLK/ADC Mode to measure VSYS/3
    //
    // Refer to $ProjectRoot/assets/images/pi_pico_w_pinout.png
    // Take the set of available pins not used by other functions, including the three pins that
    // are connected via the CYW43 Wi-Fi chip. Create [Flex] Pins out of each of the GPIO pins.
    // Put them all into the GPIO_PINS map, marking them as available
    // NOTE: All pin numbers are GPIO (BCM) Pin Numbers, not physical pin numbers
    // Take the following pins out of peripherals for use a GPIO
    unsafe {
        #[cfg(not(feature = "debug-probe"))]
        let _ = GPIO_PINS.insert(0, Available(Flex::new(peripherals.PIN_0)));
        #[cfg(not(feature = "debug-probe"))]
        let _ = GPIO_PINS.insert(1, Available(Flex::new(peripherals.PIN_1)));
        let _ = GPIO_PINS.insert(2, Available(Flex::new(peripherals.PIN_2)));
        let _ = GPIO_PINS.insert(3, Available(Flex::new(peripherals.PIN_3)));
        let _ = GPIO_PINS.insert(4, Available(Flex::new(peripherals.PIN_4)));
        let _ = GPIO_PINS.insert(5, Available(Flex::new(peripherals.PIN_5)));
        let _ = GPIO_PINS.insert(6, Available(Flex::new(peripherals.PIN_6)));
        let _ = GPIO_PINS.insert(7, Available(Flex::new(peripherals.PIN_7)));
        let _ = GPIO_PINS.insert(8, Available(Flex::new(peripherals.PIN_8)));
        let _ = GPIO_PINS.insert(9, Available(Flex::new(peripherals.PIN_9)));
        let _ = GPIO_PINS.insert(10, Available(Flex::new(peripherals.PIN_10)));
        let _ = GPIO_PINS.insert(11, Available(Flex::new(peripherals.PIN_11)));
        let _ = GPIO_PINS.insert(12, Available(Flex::new(peripherals.PIN_12)));
        let _ = GPIO_PINS.insert(13, Available(Flex::new(peripherals.PIN_13)));
        let _ = GPIO_PINS.insert(14, Available(Flex::new(peripherals.PIN_14)));
        let _ = GPIO_PINS.insert(15, Available(Flex::new(peripherals.PIN_15)));
        let _ = GPIO_PINS.insert(16, Available(Flex::new(peripherals.PIN_16)));
        let _ = GPIO_PINS.insert(17, Available(Flex::new(peripherals.PIN_17)));
        let _ = GPIO_PINS.insert(18, Available(Flex::new(peripherals.PIN_18)));
        let _ = GPIO_PINS.insert(19, Available(Flex::new(peripherals.PIN_19)));
        let _ = GPIO_PINS.insert(20, Available(Flex::new(peripherals.PIN_20)));
        let _ = GPIO_PINS.insert(21, Available(Flex::new(peripherals.PIN_21)));
        let _ = GPIO_PINS.insert(22, Available(Flex::new(peripherals.PIN_22)));
        let _ = GPIO_PINS.insert(26, Available(Flex::new(peripherals.PIN_26)));
        let _ = GPIO_PINS.insert(27, Available(Flex::new(peripherals.PIN_27)));
        let _ = GPIO_PINS.insert(28, Available(Flex::new(peripherals.PIN_28)));
    }

    // create hardware description
    let mut flash = flash::get_flash(peripherals.FLASH);
    let serial_number = flash::serial_number(&mut flash);
    static HARDWARE_DESCRIPTION: StaticCell<HardwareDescription> = StaticCell::new();
    let hw_desc = HARDWARE_DESCRIPTION.init(hardware_description(serial_number));

    #[cfg(feature = "usb")]
    let driver = Driver::new(peripherals.USB, Irqs);

    // start the flash database
    static DATABASE: StaticCell<
        Database<DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    > = StaticCell::new();
    let db = DATABASE.init(flash::db_init(flash).await);

    // Load initial config from flash
    let mut hardware_config = persistence::get_config(db).await;

    // apply the loaded config to the hardware immediately
    gpio::apply_config_change(
        &spawner,
        &HardwareConfigMessage::NewConfig(hardware_config.clone()),
        &mut hardware_config,
    )
    .await;

    #[cfg(feature = "usb")]
    let mut usb_connection = usb::start(spawner, driver, hw_desc).await;
    #[cfg(feature = "usb")]
    usb::wait_connection(&mut usb_connection, &hardware_config).await;
    #[cfg(feature = "usb")]
    usb::message_loop(&mut usb_connection, &mut hardware_config, &spawner, db).await;
}
