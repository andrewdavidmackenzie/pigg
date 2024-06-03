use std::env;

use hw::Hardware;

use crate::gpio::GPIOConfig;

mod gpio;
#[allow(dead_code)]
mod hw;

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
fn main() {
    let mut hw = hw::get();
    println!("{}", hw.descriptor().unwrap());
    println!("Pin Descriptions:");
    for pin_description in hw.pin_descriptions() {
        println!("{pin_description}")
    }

    // Load config from file or default
    let (filename, config) = match env::args().nth(1) {
        Some(config_filename) => {
            let config = GPIOConfig::load(&config_filename).unwrap();
            (Some(config_filename), config)
        }
        None => (None, GPIOConfig::default()),
    };

    match filename {
        Some(file) => {
            println!("Config loaded from file: {file}");
            println!("{config}");
        }
        None => println!("Default Config set"),
    }
    hw.apply_config(&config, |_, _| {}).unwrap();
}
