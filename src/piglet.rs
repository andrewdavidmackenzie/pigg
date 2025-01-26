#![deny(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::{
    env,
    env::current_exe,
    fs, io,
    path::{Path, PathBuf},
    process,
    process::exit,
    str::FromStr,
    time::Duration,
};

use anyhow::Context;
use clap::{Arg, ArgMatches};
use env_logger::{Builder, Target};
#[cfg(all(feature = "iroh", feature = "tcp"))]
use futures::FutureExt;
use log::{info, trace, LevelFilter};
#[cfg(all(feature = "discovery", feature = "tcp"))]
use mdns_sd::{ServiceDaemon, ServiceInfo};
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use sysinfo::{Process, System};

#[cfg(feature = "iroh")]
use crate::device::iroh_device;
#[cfg(feature = "tcp")]
use crate::device::tcp_device;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use crate::hw_definition::description::TCP_MDNS_SERVICE_TYPE;

/// Module for performing the network transfer of config and events between GUI and piglet
mod device;
/// Module for interacting with the GPIO hardware
mod hw;
/// Module that defines the structs shared back and fore between GUI and piglet/porky
mod hw_definition;
mod net;
/// Module for persisting configs across runs
mod persistence;

const SERVICE_NAME: &str = "net.mackenzie-serres.pigg.piglet";

#[cfg(any(feature = "iroh", feature = "tcp"))]
/// The [ListenerInfo] struct captures information about network connections the instance of
/// `piglet` is listening on, that can be used with `piggui` to start a remote GPIO session
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

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_matches();
    let exec_path = current_exe()?;

    manage_service(&exec_path, &matches)?;

    let info_path = check_unique(&exec_path)?;

    run_service(&info_path, &matches, exec_path).await
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
async fn run_service(
    info_path: &Path,
    matches: &ArgMatches,
    exec_path: PathBuf,
) -> anyhow::Result<()> {
    setup_logging(matches);

    let hw = hw::driver::get();
    info!("\n{}", hw.description()?.details);

    // Get the boot config for the hardware
    #[allow(unused_mut)]
    let mut hardware_config = persistence::get_config(matches, &exec_path).await;

    // Apply the initial config to the hardware, whatever it is
    hw.apply_config(&hardware_config, |bcm_pin_number, level_change| {
        info!("Pin #{bcm_pin_number} changed level to '{level_change}'")
    })
    .await?;
    trace!("Configuration applied to hardware");

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let listener_info = ListenerInfo {
        #[cfg(feature = "iroh")]
        iroh_info: iroh_device::get_device().await?,
        #[cfg(feature = "tcp")]
        tcp_info: tcp_device::get_device().await?,
    };

    // write the info about the node to the info_path file for use in piggui
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    persistence::write_info_file(info_path, &listener_info)?;

    #[cfg(any(feature = "iroh", feature = "tcp"))]
    let desc = hw.description()?;
    #[cfg(any(feature = "iroh", feature = "tcp"))]
    println!("Serial Number: {}", desc.details.serial);

    // Then listen for remote connections and "serve" them
    #[cfg(all(feature = "tcp", not(feature = "iroh")))]
    if let Some(mut listener) = listener_info.tcp_info.listener {
        #[cfg(feature = "discovery")]
        // The key string in TXT properties is case-insensitive.
        let properties = [
            ("Serial", &desc.details.serial as &str),
            ("Model", &desc.details.model as &str),
            ("AppName", env!("CARGO_BIN_NAME")),
            ("AppVersion", env!("CARGO_PKG_VERSION")),
        ];

        #[cfg(feature = "discovery")]
        let (service_info, service_daemon) = register_mdns(
            TCP_MDNS_SERVICE_TYPE,
            listener_info.tcp_info.port,
            &desc.details.serial,
            &properties,
        )?;

        loop {
            println!("Waiting for TCP connection");
            if let Ok(stream) =
                tcp_device::accept_connection(&mut listener, &desc, hardware_config.clone()).await
            {
                println!("Connected via TCP");
                let _ =
                    tcp_device::tcp_message_loop(stream, &mut hardware_config, &exec_path, &mut hw)
                        .await;
            }
        }
    }

    #[cfg(all(feature = "iroh", not(feature = "tcp")))]
    if let Some(endpoint) = listener_info.iroh_info.endpoint {
        loop {
            println!("Waiting for Iroh connection");
            if let Ok(connection) =
                iroh_device::accept_connection(&endpoint, &desc, hardware_config.clone()).await
            {
                println!("Connected via Iroh");
                let _ = iroh_device::iroh_message_loop(
                    connection,
                    &mut hardware_config,
                    &exec_path,
                    &mut hw,
                )
                .await;
            }
        }
    }

    // loop forever selecting the next connection made and then process those messages
    #[cfg(all(feature = "iroh", feature = "tcp"))]
    if let (Some(mut tcp_listener), Some(iroh_endpoint)) = (
        listener_info.tcp_info.listener,
        listener_info.iroh_info.endpoint,
    ) {
        #[cfg(feature = "discovery")]
        // The key string in TXT properties is case-insensitive.
        let properties = [
            ("Serial", &desc.details.serial as &str),
            ("Model", &desc.details.model as &str),
            ("AppName", env!("CARGO_BIN_NAME")),
            ("AppVersion", env!("CARGO_PKG_VERSION")),
            ("IrohNodeID", &listener_info.iroh_info.nodeid.to_string()),
            (
                "IrohRelayURL",
                &listener_info.iroh_info.relay_url.to_string(),
            ),
        ];

        #[cfg(feature = "discovery")]
        let (service_info, service_daemon) = register_mdns(
            TCP_MDNS_SERVICE_TYPE,
            listener_info.tcp_info.port,
            &desc.details.serial,
            &properties,
        )?;

        loop {
            println!("Waiting for Iroh or TCP connection");
            let fused_tcp =
                tcp_device::accept_connection(&mut tcp_listener, &desc, hardware_config.clone())
                    .fuse();
            let fused_iroh =
                iroh_device::accept_connection(&iroh_endpoint, &desc, hardware_config.clone())
                    .fuse();

            futures::pin_mut!(fused_tcp, fused_iroh);

            futures::select! {
                tcp_stream = fused_tcp => {
                    println!("Connected via Tcp");
                    let _ = tcp_device::tcp_message_loop(tcp_stream?, &mut hardware_config, &exec_path, hw).await;
                },
                iroh_connection = fused_iroh => {
                    println!("Connected via Iroh");
                    let _ =  iroh_device::iroh_message_loop(iroh_connection?, &mut hardware_config, &exec_path, hw).await;
                }
                complete => {}
            }
            println!("Disconnected");
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

        #[cfg(any(feature = "iroh", feature = "tcp"))]
        // If we can find the path to the executable - look for the info file
        if let Some(path) = process.exe() {
            let info_path = path.with_file_name("piglet.info");
            if info_path.exists() {
                println!("You can use the following info to connect to it:");
                println!("{}", fs::read_to_string(info_path)?);
            }
        }

        exit(1);
    }

    // remove any leftover file from a previous execution - ignore any failure
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

#[cfg(all(feature = "discovery", feature = "tcp"))]
/// Register a mDNS service so we can get discovered
fn register_mdns(
    service_type: &str,
    port: u16,
    serial_number: &str,
    properties: &[(&str, &str)],
) -> anyhow::Result<(ServiceInfo, ServiceDaemon)> {
    let service_daemon = ServiceDaemon::new().context("Could not create service daemon")?;

    let hostname = "host1".to_string(); // TODO what to put here?
    let service_hostname = format!("{}.local.", hostname);

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
        "Registered piglet (instance #{}) with mDNS: {}",
        serial_number, service_type
    );

    Ok((service_info, service_daemon))
}

/*
#[cfg(feature = "discovery")]
/// Unregister this device from mDNS prior to exit
fn unregister_mdns(service_info: ServiceInfo, service_daemon: ServiceDaemon) -> anyhow::Result<()> {
    let service_fullname = service_info.get_fullname().to_string();
    let receiver = service_daemon.unregister(&service_fullname)?;
    while let Ok(event) = receiver.recv() {
        println!("unregister result: {:?}", &event);
    }

    Ok(())
}
 */
