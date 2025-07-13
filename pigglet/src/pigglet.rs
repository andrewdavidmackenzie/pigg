#![deny(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[cfg(all(feature = "discovery", feature = "tcp"))]
use anyhow::Context;
use clap::{Arg, ArgMatches};
use env_logger::{Builder, Target};
#[cfg(all(feature = "iroh", feature = "tcp"))]
use futures::FutureExt;
use log::{info, trace, LevelFilter};
#[cfg(all(feature = "discovery", feature = "tcp"))]
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::{env, env::current_exe, path::PathBuf, process, str::FromStr};

#[cfg(feature = "iroh")]
use crate::device_net::iroh_device;
#[cfg(feature = "tcp")]
use crate::device_net::tcp_device;
use anyhow::anyhow;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
use piggpio::{check_unique, get_hardware, write_info_file, HW, PIGG_INFO_FILENAME};

/// Module for handling pigglet config files
mod config;
/// Module for performing the network transfer of config and events between GUI and pigglet
mod device_net;
mod service;

const SERVICE_NAME: &str = "net.mackenzie-serres.pigg.pigglet";

/// The [ListenerInfo] struct captures information about network connections the instance of
/// `pigglet` is listening on, that can be used with `piggui` to start a remote GPIO session
struct ListenerInfo {
    pub pid: u32,
    #[cfg(feature = "iroh")]
    pub iroh_info: iroh_device::IrohDevice,
    #[cfg(feature = "tcp")]
    pub tcp_info: tcp_device::TcpDevice,
}

impl std::fmt::Display for ListenerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PID: {}", self.pid)?;

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

    // Handle any system service installing/uninstalling - this may exit
    service::manage_service(&exec_path, &matches)?;
    setup_logging(&matches);
    run(&matches, exec_path).await
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

/// Run pigglet  - this could be interactively by a user in the foreground or
/// started by the system as a user service, in the background - use logging for output from here on
#[allow(unused_variables)]
async fn run(matches: &ArgMatches, exec_path: PathBuf) -> anyhow::Result<()> {
    let listener_info = ListenerInfo {
        pid: process::id(),
        #[cfg(feature = "iroh")]
        iroh_info: iroh_device::get_device().await?,
        #[cfg(feature = "tcp")]
        tcp_info: tcp_device::get_device().await?,
    };

    check_unique(&["pigglet"], PIGG_INFO_FILENAME)?;
    write_info_file("pigglet\n{listener_info}")?;

    if let Ok(Some(mut hw)) = get_hardware(&format!("pigglet\n{listener_info}")) {
        let desc = HW::description("pigglet").clone();
        info!("\n{}", desc.details);

        // Get the boot config for the hardware
        #[allow(unused_mut)]
        let mut hardware_config = config::get_config(matches, &exec_path).await;

        // Apply the initial config to the hardware, whatever it is
        hw.apply_config(&hardware_config, |bcm_pin_number, level_change| {
            info!("Pin #{bcm_pin_number} changed level to '{level_change}'")
        })
        .await?;
        trace!("Configuration applied to hardware");

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
        Err(anyhow!("Could not get access to GPIO hardware"))
    }
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

    // This will process and exit immediately for "--help", "--version"
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
