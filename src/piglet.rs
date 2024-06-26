use crate::hw::{BCMPinNumber, PinLevel};
use clap::{Arg, ArgMatches, Command};
use env_logger::Builder;
use hw::config::HardwareConfig;
use hw::Hardware;
use log::{info, trace, LevelFilter};
use std::str::FromStr;
use std::time::Duration;
use std::{env, thread};

mod hw;

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
fn main() {
    let matches = get_matches();

    setup_logging(&matches);

    let mut hw = hw::get();
    info!("\n{}", hw.description().unwrap().details);
    trace!("Pin Descriptions:");
    for pin_description in hw.description().unwrap().pins.iter() {
        trace!("{pin_description}")
    }

    // Load any config file specified on the command line, or else the default
    let config = match matches.get_one::<String>("config-file") {
        Some(config_filename) => {
            let config = HardwareConfig::load(config_filename).unwrap();
            info!("Config loaded from file: {config_filename}");
            trace!("{config}");
            config
        }
        None => {
            info!("Default Config loaded");
            HardwareConfig::default()
        }
    };

    hw.apply_config(&config, input_level_changed).unwrap();
    trace!("Configuration applied to hardware");
    thread::sleep(Duration::from_secs(60));
}

/// Callback function that is called when an input changes level
fn input_level_changed(bcm_pin_number: BCMPinNumber, level: PinLevel) {
    info!("Pin #{bcm_pin_number} changed level to '{level}'");
}

/// Setup logging with the requested verbosity level
fn setup_logging(matches: &ArgMatches) {
    let default = String::from("error");
    let verbosity_option = matches.get_one::<String>("verbosity");
    let verbosity = verbosity_option.unwrap_or(&default);
    let level = LevelFilter::from_str(verbosity).unwrap_or(LevelFilter::Error);
    let mut builder = Builder::from_default_env();
    builder.filter_level(level).init();
}

/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about(
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    );

    let app = app.arg(
        Arg::new("verbosity")
            .short('v')
            .long("verbosity")
            .num_args(1)
            .number_of_values(1)
            .value_name("VERBOSITY_LEVEL")
            .help(
                "Set verbosity level for output (trace, debug, info, warn, error (default), off)",
            ),
    );

    let app = app.arg(
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}
