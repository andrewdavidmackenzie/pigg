#![no_std]
#![no_main]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use crate::flash::DbFlash;
use crate::gpio::Gpio;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};
use crate::pin_descriptions::PIN_DESCRIPTIONS;
use core::str;
use defmt::error;
use ekv::Database;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{Blocking, Flash};
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

pub static HARDWARE_EVENT_CHANNEL: Channel<ThreadModeRawMutex, HardwareConfigMessage, 16> =
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

    let mut gpio = Gpio::new(
        peripherals.PIN_0,
        peripherals.PIN_1,
        peripherals.PIN_2,
        peripherals.PIN_3,
        peripherals.PIN_4,
        peripherals.PIN_5,
        peripherals.PIN_6,
        peripherals.PIN_7,
        peripherals.PIN_8,
        peripherals.PIN_9,
        peripherals.PIN_10,
        peripherals.PIN_11,
        peripherals.PIN_12,
        peripherals.PIN_13,
        peripherals.PIN_14,
        peripherals.PIN_15,
        peripherals.PIN_16,
        peripherals.PIN_17,
        peripherals.PIN_18,
        peripherals.PIN_19,
        peripherals.PIN_20,
        peripherals.PIN_21,
        peripherals.PIN_22,
        peripherals.PIN_26,
        peripherals.PIN_27,
        peripherals.PIN_28,
    );

    // create hardware description
    let mut flash = flash::get_flash(peripherals.FLASH);
    let serial_number = flash::serial_number(&mut flash);
    static HARDWARE_DESCRIPTION: StaticCell<HardwareDescription> = StaticCell::new();
    let hw_desc = HARDWARE_DESCRIPTION.init(hardware_description(serial_number));

    let driver = Driver::new(peripherals.USB, Irqs);

    // start the flash database
    static DATABASE: StaticCell<
        Database<DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    > = StaticCell::new();
    let db = DATABASE.init(flash::db_init(flash).await);

    // Load initial config from flash
    let mut hardware_config = persistence::get_config(db).await;

    // apply the loaded config to the hardware immediately
    gpio.apply_config_change(
        &spawner,
        &HardwareConfigMessage::NewConfig(hardware_config.clone()),
        &mut hardware_config,
    )
    .await;

    let mut usb_connection = usb::start(spawner, driver, hw_desc).await;

    loop {
        if usb::wait_connection(&mut usb_connection, &hardware_config)
            .await
            .is_err()
        {
            error!("Could not establish USB connection");
        } else {
            let _ = usb::message_loop(
                &mut gpio,
                &mut usb_connection,
                &mut hardware_config,
                &spawner,
                db,
            )
            .await;
        }
    }
}
