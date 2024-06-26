use clap::{Arg, ArgMatches, Command};
use env_logger::Builder;
use hw::config::HardwareConfig;
use hw::Hardware;
use log::{info, trace, LevelFilter};
use std::env;
use std::str::FromStr;

mod hw;

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
fn main() {
    let matches = get_matches();

    // Setup logging with the requested verbosity level
    let default = String::from("error");
    let verbosity_option = matches.get_one::<String>("verbosity");
    let verbosity = verbosity_option.unwrap_or(&default);
    let level = LevelFilter::from_str(verbosity).unwrap_or(LevelFilter::Error);
    let mut builder = Builder::from_default_env();
    builder.filter_level(level).init();

    let mut hw = hw::get();
    info!("{}", hw.description().unwrap().details);
    trace!("Pin Descriptions:");
    for pin_description in hw.description().unwrap().pins.iter() {
        trace!("{pin_description}")
    }

    // Load config from file or default
    let (filename, config) = match matches.get_one::<String>("config") {
        Some(config_filename) => {
            let config = HardwareConfig::load(&config_filename).unwrap();
            (Some(config_filename), config)
        }
        None => (None, HardwareConfig::default()),
    };

    match filename {
        Some(file) => {
            info!("Config loaded from file: {file}");
            trace!("{config}");
        }
        None => info!("Default Config set"),
    }
    hw.apply_config(&config, |_, _| {}).unwrap();
}

/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = Command::new(env!("CARGO_PKG_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.arg(
        Arg::new("verbosity")
            .short('v')
            .long("verbosity")
            .num_args(0..1)
            .number_of_values(1)
            .value_name("VERBOSITY_LEVEL")
            .help(
                "Set verbosity level for output (trace, debug, info, warn, error (default), off)",
            ),
    );

    let app = app.arg(
        Arg::new("config")
            .num_args(1)
            .help("the path of a '.pigg' config file to load"),
    );

    app.get_matches()
}
