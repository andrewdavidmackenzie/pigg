#![deny(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::{
    env,
    env::current_exe,
    fmt, fs, io,
    path::{Path, PathBuf},
    process,
    process::exit,
    str::FromStr,
    time::Duration,
};

use anyhow::Context;
use clap::{Arg, ArgMatches};
use log::{info, trace};
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use sysinfo::{Process, System};
use tracing::Level;
use tracing_subscriber::filter::{Directive, LevelFilter};
use tracing_subscriber::EnvFilter;

use hw::config::HardwareConfig;
use hw::Hardware;

use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::{fs::File, io::Write};

mod hw;
#[cfg(feature = "iroh")]
mod piglet_iroh_helper;
#[cfg(feature = "tcp")]
mod piglet_tcp_helper;

const SERVICE_NAME: &str = "net.mackenzie-serres.pigg.piglet";

/// The [ListenerInfo] struct captures information about network connections the instance of
/// `piglet` is listening on, that can be used with `piggui` to start a remote GPIO session
#[derive(Serialize, Deserialize)]
struct ListenerInfo {
    #[cfg(feature = "iroh")]
    pub iroh_info: Option<piglet_iroh_helper::IrohInfo>,
    #[cfg(feature = "tcp")]
    pub tcp_info: Option<piglet_tcp_helper::TcpInfo>,
}

impl fmt::Display for ListenerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(iroh_info) = &self.iroh_info {
            writeln!(f, "{}", iroh_info)?;
        }

        if let Some(tcp_info) = &self.tcp_info {
            writeln!(f, "{}", tcp_info)?;
        }

        Ok(())
    }
}

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_matches();
    let exec_path = current_exe()?;

    manage_service(&exec_path, &matches)?;

    let info_path = check_unique(&exec_path)?;

    run_service(&info_path, &matches).await
}

/// Handle any service installation or uninstallation tasks specified on the command line
/// continue without doing anything if none were specified
fn manage_service(exec_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
    let service_name: ServiceLabel = SERVICE_NAME.parse()?;

    if matches.get_flag("uninstall") {
        uninstall_service(&service_name)?;
        exit(0);
    }

    if matches.get_flag("install") {
        install_service(&service_name, exec_path)?;
        exit(0);
    };

    Ok(())
}

/// Run piglet as a service - this could be interactively by a user in foreground or started
/// by the system as a user service, in background - use logging for output from here on
#[allow(unused_variables)]
async fn run_service(info_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
    setup_logging(matches);

    let mut hw = hw::get();
    info!("\n{}", hw.description()?.details);

    // Load any config file specified on the command line
    if let Some(config_filename) = matches.get_one::<String>("config-file") {
        let config = HardwareConfig::load(config_filename)?;
        info!("Config loaded from file: {config_filename}");
        trace!("{config}");
        hw.apply_config(&config, |bcm_pin_number, level| {
            info!("Pin #{bcm_pin_number} changed level to '{level}'")
        })?;
        trace!("Configuration applied to hardware");
    };

    let listener_info = ListenerInfo {
        iroh_info: piglet_iroh_helper::get_iroh_listener_info().await.ok(),
        tcp_info: piglet_tcp_helper::get_tcp_listener_info().await.ok(),
    };

    // write the info about the node to the info_path file for use in piggui
    write_info_file(info_path, &listener_info)?;

    // Then listen for remote connections and "serve" them
    // TODO listen to both at the same time and chose first
    #[cfg(feature = "tcp")]
    if let Some(tcp_info) = listener_info.tcp_info {
        piglet_tcp_helper::listen_tcp(tcp_info, &mut hw).await?;
    }

    #[cfg(feature = "iroh")]
    if let Some(iroh_info) = listener_info.iroh_info {
        if let Some(endpoint) = iroh_info.endpoint {
            piglet_iroh_helper::listen_iroh(&endpoint, &mut hw).await?;
        }
    }

    Ok(())
}

/// Check that this is the only instance of piglet running, both user process or system process
/// If another version is detected:
/// - print out that fact, with the process ID
/// - print out the nodeid of the instance that is running
/// - exit
fn check_unique(exec_path: &Path) -> anyhow::Result<PathBuf> {
    let exec_name = exec_path
        .file_name()
        .context("Could not get exec file name")?
        .to_str()
        .context("Could not get exec file name")?;
    let info_path = exec_path.with_file_name("piglet.info");

    let my_pid = process::id();
    let sys = System::new_all();
    let instances: Vec<&Process> = sys
        .processes_by_exact_name(exec_name.as_ref())
        .filter(|p| p.thread_kind().is_none() && p.pid().as_u32() != my_pid)
        .collect();
    if let Some(process) = instances.first() {
        println!(
            "An instance of {exec_name} is already running with PID='{}'",
            process.pid(),
        );

        // If we can find the path to the executable - look for the info file
        if let Some(path) = process.exe() {
            let info_path = path.with_file_name("piglet.info");
            if info_path.exists() {
                println!("You can use the following info to connect to it:");
                let piglet_info = fs::read_to_string(info_path)?;
                let info: ListenerInfo = serde_json::from_str(&piglet_info)?;
                println!("{}", info);
            }
        }

        exit(1);
    }

    // remove any leftover file from a previous execution - ignore any failure
    let _ = fs::remove_file(&info_path);

    Ok(info_path)
}

/// Setup logging with the requested verbosity level - or default if none was specified
fn setup_logging(matches: &ArgMatches) {
    let default: Directive = LevelFilter::from_level(Level::ERROR).into();
    let verbosity_option = matches
        .get_one::<String>("verbosity")
        .and_then(|v| Directive::from_str(v).ok());
    let directive = verbosity_option.unwrap_or(default);
    let env_filter = EnvFilter::builder()
        .with_default_directive(directive)
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

/// Parse the command line arguments using clap into a set of [ArgMatches]
fn get_matches() -> ArgMatches {
    let app = clap::Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about(
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    );

    let app = app.arg(
        Arg::new("install")
            .short('i')
            .long("install")
            .action(clap::ArgAction::SetTrue)
            .help("Install piglet as a System Service that restarts on reboot")
            .conflicts_with("uninstall"),
    );

    let app = app.arg(
        Arg::new("uninstall")
            .short('u')
            .long("uninstall")
            .action(clap::ArgAction::SetTrue)
            .help("Uninstall any piglet System Service")
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
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// Write a [ListenerInfo] file that captures information that can be used to connect to piglet
fn write_info_file(info_path: &Path, listener_info: &ListenerInfo) -> anyhow::Result<()> {
    let mut output = File::create(info_path)?;
    write!(output, "{}", serde_json::to_string(listener_info)?)?;
    info!("Info file written at: {info_path:?}");
    Ok(())
}

/// Get a [ServiceManager] instance to use to install or remove system services
fn get_service_manager() -> Result<Box<dyn ServiceManager>, io::Error> {
    // Get generic service by detecting what is available on the platform
    let manager = <dyn ServiceManager>::native()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not create ServiceManager"))?;

    Ok(manager)
}

/// Install the binary as a user level service and then start it
fn install_service(service_name: &ServiceLabel, exec_path: &Path) -> Result<(), io::Error> {
    let manager = get_service_manager()?;
    // Run from dir where exec is for now, so it should find the config file in ancestors path
    let exec_dir = exec_path
        .parent()
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not get exec dir",
        ))?
        .to_path_buf();

    // Install our service using the underlying service management platform
    manager.install(ServiceInstallCtx {
        label: service_name.clone(),
        program: exec_path.to_path_buf(),
        args: vec![],
        contents: None, // Optional String for system-specific service content.
        username: None, // Optional String for alternative user to run service.
        working_directory: Some(exec_dir),
        environment: None, // Optional list of environment variables to supply the service process.
        autostart: true,
    })?;

    // Start our service using the underlying service management platform
    manager.start(ServiceStartCtx {
        label: service_name.clone(),
    })?;

    println!(
        "'service '{}' ('{}') installed and started",
        service_name,
        exec_path.display()
    );

    Ok(())
}

/// Stop any running instance of the service, then uninstall it
fn uninstall_service(service_name: &ServiceLabel) -> Result<(), io::Error> {
    let manager = get_service_manager()?;

    // Stop our service using the underlying service management platform
    manager.stop(ServiceStopCtx {
        label: service_name.clone(),
    })?;

    println!(
        "service '{}' stopped. Waiting for 10s before uninstalling",
        service_name
    );
    std::thread::sleep(Duration::from_secs(10));

    // Uninstall our service using the underlying service management platform
    manager.uninstall(ServiceUninstallCtx {
        label: service_name.clone(),
    })?;

    println!("service '{}' uninstalled", service_name);

    Ok(())
}

#[cfg(any(feature = "iroh", feature = "tcp"))]
#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::ListenerInfo;
    use tempfile::tempdir;

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file() {
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.info");
        let nodeid =
            iroh_net::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
                .expect("Could not create nodeid");
        let local_addrs = "79.154.163.213:58604 192.168.1.77:58604";
        let relay_url = iroh_net::relay::RelayUrl::from_str("https://euw1-1.relay.iroh.network./ ")
            .expect("Could not create Relay URL");

        let info = ListenerInfo {
            iroh_info: Some(crate::piglet_iroh_helper::IrohInfo {
                nodeid,
                local_addrs: local_addrs.to_string(),
                relay_url,
                alpn: "".to_string(),
                endpoint: None,
            }),
            tcp_info: None,
        };

        super::write_info_file(&test_file, &info).expect("Writing info file failed");
        assert!(test_file.exists(), "File was not created as expected");
        let piglet_info = fs::read_to_string(test_file).expect("Could not read info file");
        assert!(piglet_info.contains(&nodeid.to_string()));
        let read_info: ListenerInfo =
            serde_json::from_str(&piglet_info).expect("Could not parse info file");
        assert_eq!(nodeid, read_info.iroh_info.expect("No iroh info!").nodeid);
    }

    #[cfg(feature = "iroh")]
    #[test]
    fn write_info_file_non_existent() {
        let output_dir = PathBuf::from("/foo");
        let test_file = output_dir.join("test.info");
        let nodeid =
            iroh_net::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
                .expect("Could not create nodeid");
        let local_addrs = "79.154.163.213:58604 192.168.1.77:58604";
        let relay_url = iroh_net::relay::RelayUrl::from_str("https://euw1-1.relay.iroh.network./ ")
            .expect("Could not create Relay URL");
        let info = ListenerInfo {
            iroh_info: Some(crate::piglet_iroh_helper::IrohInfo {
                nodeid,
                local_addrs: local_addrs.to_string(),
                relay_url,
                alpn: "".to_string(),
                endpoint: None,
            }),
            tcp_info: None,
        };

        assert!(super::write_info_file(&test_file, &info).is_err());
        assert!(!test_file.exists(), "File was created!");
    }
}
