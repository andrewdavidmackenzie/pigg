#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use crate::flash::DbFlash;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};
use crate::pin_descriptions::PIN_DESCRIPTIONS;
use core::str;
use defmt::{error, info};
use ekv::{Database, ReadError};
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::FLASH;
#[cfg(feature = "usb")]
use embassy_rp::peripherals::USB;
#[cfg(feature = "usb")]
use embassy_rp::usb::Driver;
#[cfg(feature = "usb")]
use embassy_rp::usb::InterruptHandler as USBInterruptHandler;
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use heapless::Vec;
use static_cell::StaticCell;

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

/// Definition of hardware structs passed back and fore between porky and the GUI
#[path = "../../src/hw_definition/mod.rs"]
mod hw_definition;

/// Functions for interacting with the Flash ROM
mod flash;

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

    // Take the following pins out of peripherals for use a GPIO
    let header_pins = gpio::HeaderPins {
        #[cfg(not(feature = "debug-probe"))]
        pin_0: peripherals.PIN_0,
        #[cfg(not(feature = "debug-probe"))]
        pin_1: peripherals.PIN_1,
        pin_2: peripherals.PIN_2,
        pin_3: peripherals.PIN_3,
        pin_4: peripherals.PIN_4,
        pin_5: peripherals.PIN_5,
        pin_6: peripherals.PIN_6,
        pin_7: peripherals.PIN_7,
        pin_8: peripherals.PIN_8,
        pin_9: peripherals.PIN_9,
        pin_10: peripherals.PIN_10,
        pin_11: peripherals.PIN_11,
        pin_12: peripherals.PIN_12,
        pin_13: peripherals.PIN_13,
        pin_14: peripherals.PIN_14,
        pin_15: peripherals.PIN_15,
        pin_16: peripherals.PIN_16,
        pin_17: peripherals.PIN_17,
        pin_18: peripherals.PIN_18,
        pin_19: peripherals.PIN_19,
        pin_20: peripherals.PIN_20,
        pin_21: peripherals.PIN_21,
        pin_22: peripherals.PIN_22,
        pin_26: peripherals.PIN_26,
        pin_27: peripherals.PIN_27,
        pin_28: peripherals.PIN_28,
    };
    gpio::setup_pins(header_pins);

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

    #[cfg(feature = "usb")]
    let watchdog = Watchdog::new(peripherals.WATCHDOG);

    #[cfg(feature = "usb")]
    usb::start(spawner, driver, hw_desc, None, db, watchdog).await;
}
