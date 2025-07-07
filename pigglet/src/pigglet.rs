#![deny(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use anyhow::Context;
use clap::{Arg, ArgMatches};
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
#[cfg(all(feature = "discovery", feature = "tcp"))]
use mdns_sd::{ServiceDaemon, ServiceInfo};
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::fs::File;
use std::{
    env,
    env::current_exe,
    fs,
    path::{Path, PathBuf},
    process,
    process::exit,
    str::FromStr,
};
use sysinfo::{Process, System};

#[cfg(feature = "iroh")]
use crate::device_net::iroh_device;
#[cfg(feature = "tcp")]
use crate::device_net::tcp_device;
use service_manager::ServiceLabel;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::io::Write;

/// Module for handling pigglet config files
mod config;
/// Module for performing the network transfer of config and events between GUI and pigglet
mod device_net;
mod service;

const SERVICE_NAME: &str = "net.mackenzie-serres.pigg.pigglet";
const PIGG_INFO_FILENAME: &str = "pigglet.info";

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// Write a [ListenerInfo] file that captures information that can be used to connect to pigglet
pub(crate) fn write_info_file(
    info_path: &Path,
    listener_info: &ListenerInfo,
) -> anyhow::Result<()> {
    let mut output = File::create(info_path)?;
    write!(output, "{listener_info}")?;
    info!("Info file written at: {info_path:?}");
    Ok(())
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// The [ListenerInfo] struct captures information about network connections the instance of
/// `pigglet` is listening on, that can be used with `piggui` to start a remote GPIO session
struct ListenerInfo {
    #[cfg(feature = "iroh")]
    pub iroh_info: iroh_device::IrohDevice,
    #[cfg(feature = "tcp")]
    pub tcp_info: tcp_device::TcpDevice,
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
impl std::fmt::Display for ListenerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "iroh")]
        writeln!(f, "{}", self.iroh_info)?;

        #[cfg(feature = "tcp")]
        writeln!(f, "{}", self.tcp_info)?;

        Ok(())
    }
}

/// Pigglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from a file and
/// over the network.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_matches();
    let exec_path = current_exe()?;

    manage_service(&exec_path, &matches)?;

    let info_path = check_unique(&["pigglet", "piggui"])?;

    service::run_service(&info_path, &matches, exec_path).await
}

/// Handle any service installation or uninstallation tasks specified on the command line
/// continue without doing anything if none were specified
fn manage_service(exec_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
    let service_name: ServiceLabel = SERVICE_NAME.parse()?;

    if matches.get_flag("uninstall") {
        service::uninstall_service(&service_name)?;
        exit(0);
    }

    if matches.get_flag("install") {
        service::install_service(&service_name, exec_path)?;
        exit(0);
    };

    Ok(())
}

/// Check that this is the only instance of the process running (user or service)
/// If another version is detected:
/// - print out that fact, with the process ID
/// - print out the nodeid of the instance that is running
/// - exit
fn check_unique(names: &[&str]) -> anyhow::Result<PathBuf> {
    let my_pid = process::id();
    let sys = System::new_all();
    for process_name in names {
        let instances: Vec<&Process> = sys
            .processes_by_exact_name(process_name.as_ref())
            .filter(|p| p.thread_kind().is_none() && p.pid().as_u32() != my_pid)
            .collect();
        if let Some(process) = instances.first() {
            println!(
                "An instance of {process_name} is already running with PID='{}'",
                process.pid(),
            );

            #[cfg(any(feature = "iroh", feature = "tcp"))]
            // If we can find the path to the executable - look for the info file
            if let Some(path) = process.exe() {
                let info_path = path.with_file_name(PIGG_INFO_FILENAME);
                if info_path.exists() {
                    println!("You can use the following info to connect to it:");
                    println!("{}", fs::read_to_string(info_path)?);
                }
            }

            exit(1);
        }
    }

    // remove any leftover file from a previous execution - ignore any failure
    let exec_path = current_exe()?;
    let info_path = exec_path.with_file_name(PIGG_INFO_FILENAME);
    let _ = fs::remove_file(&info_path);

    Ok(info_path)
}

/// Setup logging to StdOut with the requested verbosity level - or default (ERROR) if no valid
/// debug level was specified
fn setup_logging(matches: &ArgMatches) {
    let default_verbosity = "error".to_string();
    let verbosity = matches
        .get_one::<String>("verbosity")
        .unwrap_or(&default_verbosity);
    let level = LevelFilter::from_str(verbosity).unwrap_or(LevelFilter::Error);
    let mut builder = Builder::from_default_env();
    builder.filter_level(level).target(Target::Stdout).init();
}

/// Parse the command line arguments using clap into a set of [ArgMatches]
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about(
        "'pigglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    );

    let app = app.arg(
        Arg::new("install")
            .short('i')
            .long("install")
            .action(clap::ArgAction::SetTrue)
            .help("Install pigglet as a System Service that restarts on reboot")
            .conflicts_with("uninstall"),
    );

    let app = app.arg(
        Arg::new("uninstall")
            .short('u')
            .long("uninstall")
            .action(clap::ArgAction::SetTrue)
            .help("Uninstall any pigglet System Service")
            .conflicts_with("install"),
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
        Arg::new("config")
            .short('c')
            .long("config")
            .num_args(1)
            .number_of_values(1)
            .value_name("Config File")
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}

#[cfg(all(feature = "discovery", feature = "tcp"))]
/// Register a mDNS service so we can get discovered
fn register_mdns(
    service_type: &str,
    port: u16,
    serial_number: &str,
    properties: &[(&str, &str)],
) -> anyhow::Result<(ServiceInfo, ServiceDaemon)> {
    let service_daemon = ServiceDaemon::new().context("Could not create service daemon")?;

    let service_hostname = format!("{serial_number}.local.");

    // Register a service.
    let service_info = ServiceInfo::new(
        service_type,
        serial_number,
        &service_hostname,
        "",
        port,
        properties,
    )
    .context("Could not create mDNS ServiceInfo")?
    .enable_addr_auto();

    service_daemon
        .register(service_info.clone())
        .context("Could not register mDNS daemon")?;

    println!(
        "Registered pigglet with mDNS:\n\tInstance: {serial_number}\n\tHostname: {service_hostname}\n\tService Type: {service_type}"
    );

    Ok((service_info, service_daemon))
}

#[cfg(test)]
mod test {
    use crate::ListenerInfo;
    use iroh::{NodeId, RelayUrl};
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::tempdir;

    fn listener_info(nodeid: &NodeId, relay_url_str: &str) -> ListenerInfo {
        ListenerInfo {
            iroh_info: crate::iroh_device::IrohDevice {
                nodeid: *nodeid,
                relay_url: RelayUrl::from_str(relay_url_str).expect("Could not create Relay URL"),
                endpoint: None,
            },

            #[cfg(feature = "tcp")]
            tcp_info: crate::tcp_device::TcpDevice {
                ip: std::net::IpAddr::from_str("10.0.0.0").expect("Could not parse IpAddr"),
                port: 9001,
                listener: None,
            },
        }
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file() {
        let output_dir = tempdir().expect("Could not create a tempdir").keep();
        let test_file = output_dir.join("test.info");
        let nodeid = iroh::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
            .expect("Could not create nodeid");
        super::write_info_file(
            &test_file,
            &listener_info(&nodeid, "https://euw1-1.relay.iroh.network./ "),
        )
        .expect("Writing info file failed");
        assert!(test_file.exists(), "File was not created as expected");
        let pigglet_info = fs::read_to_string(test_file).expect("Could not read info file");
        println!("pigglet_info: {pigglet_info}");
        assert!(pigglet_info.contains(&nodeid.to_string()))
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file_non_existent() {
        let output_dir = PathBuf::from("/foo");
        let test_file = output_dir.join("test.info");
        let nodeid = iroh::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
            .expect("Could not create nodeid");
        assert!(super::write_info_file(
            &test_file,
            &listener_info(&nodeid, "https://euw1-1.relay.iroh.network./ "),
        )
        .is_err());
        assert!(!test_file.exists(), "File was created!");
    }
}
