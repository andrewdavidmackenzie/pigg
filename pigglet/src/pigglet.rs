#![deny(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use anyhow::anyhow;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use anyhow::Context;
use clap::{Arg, ArgMatches};
use env_logger::{Builder, Target};
#[cfg(all(feature = "iroh", feature = "tcp"))]
use futures::FutureExt;
use log::{info, trace, LevelFilter};
#[cfg(all(feature = "discovery", feature = "tcp"))]
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::{env, env::current_exe, fs, path::PathBuf, process, process::exit, str::FromStr};
use sysinfo::{Process, System};

use piggpio::get_hardware;

#[cfg(feature = "iroh")]
use crate::device_net::iroh_device;
#[cfg(feature = "tcp")]
use crate::device_net::tcp_device;
use crate::instance::InstanceInfo;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;

/// Module for handling pigglet config files
mod config;
/// Module for performing the network transfer of config and events between GUI and pigglet
mod device_net;
mod instance;
mod service;

const PIGG_INFO_FILENAME: &str = "pigglet.info";

/// Pigglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from a file and
/// over the network.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_matches();
    let exec_path = current_exe()?;

    service::manage(&exec_path, &matches)?;

    match check_unique("pigglet") {
        Ok(_) => run(&matches, exec_path).await,
        Err(None) => {
            println!("There is another instance of pigglet running, but we couldn't get more information");
            exit(1);
        }
        Err(Some(listener_info)) => {
            println!(
                "An instance of {} is already running with PID='{}'",
                listener_info.process_name, listener_info.pid,
            );
            println!("You can use the following info to connect to it:\n{listener_info}");

            exit(1);
        }
    }
}

/// Run pigglet - interactively by a user in the foreground or started by the system as a
/// user service, in the background - use logging for output from here on
#[allow(unused_variables)]
async fn run(matches: &ArgMatches, exec_path: PathBuf) -> anyhow::Result<()> {
    // We'll create the info file in the directory where we are running
    let info_path = exec_path.with_file_name(PIGG_INFO_FILENAME);

    // remove any leftover file from a previous execution - ignore any failure
    let _ = fs::remove_file(&info_path);

    setup_logging(matches);

    if let Some(mut hw) = get_hardware() {
        info!("\n{}", hw.description().details);

        // Get the boot config for the hardware
        #[allow(unused_mut)]
        let mut hardware_config = config::get_config(matches, &exec_path).await;

        // Apply the initial config to the hardware, whatever it is
        hw.apply_config(&hardware_config, |bcm_pin_number, level_change| {
            info!("Pin #{bcm_pin_number} changed level to '{level_change}'")
        })
        .await?;
        trace!("Configuration applied to hardware");

        let listener_info = InstanceInfo {
            process_name: "pigglet".to_string(),
            pid: process::id(),
            #[cfg(feature = "iroh")]
            iroh_info: iroh_device::get_device().await?,
            #[cfg(feature = "tcp")]
            tcp_info: tcp_device::get_device().await?,
        };

        // write the info about the node to the info_path file for use in piggui
        listener_info.write_to_file(&info_path)?;

        #[cfg(any(feature = "iroh", feature = "tcp"))]
        let desc = hw.description().clone();
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
                    tcp_device::accept_connection(&mut listener, &desc, hardware_config.clone())
                        .await
                {
                    println!("Connection via TCP");
                    let _ = tcp_device::tcp_message_loop(
                        stream,
                        &mut hardware_config,
                        &exec_path,
                        &mut hw,
                    )
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
                    println!("Connection via Iroh");
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
                let fused_tcp = tcp_device::accept_connection(
                    &mut tcp_listener,
                    &desc,
                    hardware_config.clone(),
                )
                .fuse();
                let fused_iroh =
                    iroh_device::accept_connection(&iroh_endpoint, &desc, hardware_config.clone())
                        .fuse();

                futures::pin_mut!(fused_tcp, fused_iroh);

                futures::select! {
                    tcp_stream = fused_tcp => {
                        println!("Connection via Tcp");
                        let _ = tcp_device::tcp_message_loop(tcp_stream?, &mut hardware_config, &exec_path, &mut hw).await;
                    },
                    iroh_connection = fused_iroh => {
                        println!("Connection via Iroh");
                        let _ =  iroh_device::iroh_message_loop(iroh_connection?, &mut hardware_config, &exec_path, &mut hw).await;
                    }
                    complete => {}
                }
                println!("Disconnected");
            }
        }

        Ok(())
    } else {
        Err(anyhow!("Could not get hardware"))
    }
}

/// Check that this is the only instance of a process running, both user process or system process
/// If no other instance is detected, return Ok
/// If another version is detected, return:
///     - Err(Some([InstanceInfo])) if the info file could be read and parsed
///     - Err(None) if the file could not be read, or the contents not parsed to a [InstanceInfo]
#[allow(clippy::result_large_err)]
fn check_unique(process_name: &str) -> anyhow::Result<(), Option<InstanceInfo>> {
    let my_pid = process::id();
    let sys = System::new_all();
    let instances: Vec<&Process> = sys
        .processes_by_exact_name(process_name.as_ref())
        .filter(|p| p.thread_kind().is_none() && p.pid().as_u32() != my_pid)
        .collect();
    if let Some(process) = instances.first() {
        // If we can find the path to the executable - load for the info file
        if let Some(path) = process.exe() {
            let info_path = path.with_file_name(PIGG_INFO_FILENAME);
            return Err(Some(
                InstanceInfo::load_from_file(info_path).map_err(|_| None)?,
            ));
        }

        return Err(None);
    }

    Ok(())
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
