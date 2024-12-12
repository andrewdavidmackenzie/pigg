use crate::hw_definition::config::HardwareConfig;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use crate::ListenerInfo;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use anyhow::Context;
use clap::ArgMatches;
use log::{info, trace};
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::fs::File;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::io::Write;
use std::path::Path;

/// Get the initial [HardwareConfig] determined following:
/// - A config file specified on the command line, or
/// - A config file saved from a previous run
/// - The default (empty) config
pub(crate) async fn get_config(matches: &ArgMatches, exec_path: &Path) -> HardwareConfig {
    // A config file specified on the command line overrides any config file from previous run
    let config_file = matches.get_one::<String>("config-file");

    // Load any config file specified on the command line
    match config_file {
        Some(config_filename) => match HardwareConfig::load(config_filename) {
            Ok(config) => {
                println!("Config loaded from file: {config_filename}");
                trace!("{config}");
                config
            }
            Err(_) => {
                info!("Loaded default config");
                HardwareConfig::default()
            }
        },
        None => {
            // look for config file from last run
            let last_run_filename = exec_path.with_file_name(".piglet_config.json");
            match HardwareConfig::load(&last_run_filename.to_string_lossy()) {
                Ok(config) => {
                    println!("Config loaded from file: {}", last_run_filename.display());
                    trace!("{config}");
                    config
                }
                Err(_) => {
                    println!("Loaded default config");
                    HardwareConfig::default()
                }
            }
        }
    }
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// Save the config to a file that will be picked up on restart
pub(crate) async fn store_config(
    hardware_config: &HardwareConfig,
    exec_path: &Path,
) -> anyhow::Result<()> {
    let last_run_filename = exec_path.with_file_name(".piglet_config.json");
    hardware_config
        .save(&last_run_filename.to_string_lossy())
        .with_context(|| "Saving hardware config")?;
    Ok(())
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// Write a [ListenerInfo] file that captures information that can be used to connect to piglet
pub(crate) fn write_info_file(
    info_path: &Path,
    listener_info: &ListenerInfo,
) -> anyhow::Result<()> {
    let mut output = File::create(info_path)?;
    write!(output, "{}", listener_info)?;
    info!("Info file written at: {info_path:?}");
    Ok(())
}

#[cfg(feature = "iroh")]
#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::{persistence, ListenerInfo};
    use tempfile::tempdir;

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file() {
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.info");
        let nodeid =
            iroh_net::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
                .expect("Could not create nodeid");
        let relay_url = iroh_net::relay::RelayUrl::from_str("https://euw1-1.relay.iroh.network./ ")
            .expect("Could not create Relay URL");

        let info = ListenerInfo {
            iroh_info: crate::iroh_device::IrohDevice {
                nodeid,
                relay_url,
                endpoint: None,
            },
            #[cfg(feature = "tcp")]
            tcp_info: crate::tcp_device::TcpDevice {
                ip: std::net::IpAddr::from_str("10.0.0.0").expect("Could not parse IpAddr"),
                port: 9001,
                listener: None,
            },
        };

        persistence::write_info_file(&test_file, &info).expect("Writing info file failed");
        assert!(test_file.exists(), "File was not created as expected");
        let piglet_info = fs::read_to_string(test_file).expect("Could not read info file");
        assert!(piglet_info.contains(&nodeid.to_string()))
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file_non_existent() {
        let output_dir = PathBuf::from("/foo");
        let test_file = output_dir.join("test.info");
        let nodeid =
            iroh_net::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
                .expect("Could not create nodeid");
        let relay_url = iroh_net::relay::RelayUrl::from_str("https://euw1-1.relay.iroh.network./ ")
            .expect("Could not create Relay URL");
        let info = ListenerInfo {
            iroh_info: crate::iroh_device::IrohDevice {
                nodeid,
                relay_url,
                endpoint: None,
            },
            #[cfg(feature = "tcp")]
            tcp_info: crate::tcp_device::TcpDevice {
                ip: std::net::IpAddr::from_str("10.0.0.0").expect("Could not parse IpAddr"),
                port: 9001,
                listener: None,
            },
        };

        assert!(persistence::write_info_file(&test_file, &info).is_err());
        assert!(!test_file.exists(), "File was created!");
    }
}
