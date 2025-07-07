use crate::device_net::{iroh_device, tcp_device};
use crate::{config, register_mdns, setup_logging, write_info_file, ListenerInfo, SERVICE_NAME};
use anyhow::anyhow;
use clap::ArgMatches;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use futures::FutureExt;
use log::{info, trace};
#[cfg(all(feature = "discovery", feature = "tcp"))]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
use piggpio::get_hardware;
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;

/// Handle any service installation or uninstallation tasks specified on the command line
/// continue without doing anything if none were specified
pub fn manage_service(exec_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
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

/// Run pigglet as a service - this could be interactively by a user in the foreground or
/// started by the system as a user service, in the background - use logging for output from here on
#[allow(unused_variables)]
pub async fn run_service(
    info_path: &Path,
    matches: &ArgMatches,
    exec_path: PathBuf,
) -> anyhow::Result<()> {
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

        #[cfg(any(feature = "iroh", feature = "tcp"))]
        let listener_info = ListenerInfo {
            #[cfg(feature = "iroh")]
            iroh_info: iroh_device::get_device().await?,
            #[cfg(feature = "tcp")]
            tcp_info: tcp_device::get_device().await?,
        };

        // write the info about the node to the info_path file for use in piggui
        #[cfg(any(feature = "iroh", feature = "tcp"))]
        write_info_file(info_path, &listener_info)?;

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

/// Get a [ServiceManager] instance to use to install or remove system services
fn get_service_manager() -> Result<Box<dyn ServiceManager>, io::Error> {
    // Get generic service by detecting what is available on the platform
    let manager = <dyn ServiceManager>::native()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not create ServiceManager"))?;

    Ok(manager)
}

/// Install the binary as a user-level service and then start it
fn install_service(service_name: &ServiceLabel, exec_path: &Path) -> Result<(), io::Error> {
    let manager = get_service_manager()?;
    // Run from a dir where exec is for now, so it should find the config file in the ancestor's path
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
        username: None, // Optional String for an alternative user to run service.
        working_directory: Some(exec_dir),
        environment: None, // Optional list of environment variables to supply the service process.
        autostart: true,
        disable_restart_on_failure: false,
    })?;

    // Start our service using the underlying service management platform
    manager.start(ServiceStartCtx {
        label: service_name.clone(),
    })?;

    println!(
        "service '{}' ('{}') installed and started",
        service_name,
        exec_path.display()
    );

    #[cfg(target_os = "linux")]
    println!(
        "You can view service logs using 'sudo journalctl -u {}'",
        service_name
    );

    Ok(())
}

/// Stop any running instance of the service, then uninstall it
pub fn uninstall_service(service_name: &ServiceLabel) -> Result<(), io::Error> {
    let manager = get_service_manager()?;

    // Stop our service using the underlying service management platform
    manager.stop(ServiceStopCtx {
        label: service_name.clone(),
    })?;

    println!("service '{service_name}' stopped. Waiting for 10s before uninstalling");
    std::thread::sleep(Duration::from_secs(10));

    // Uninstall our service using the underlying service management platform
    manager.uninstall(ServiceUninstallCtx {
        label: service_name.clone(),
    })?;

    println!("service '{service_name}' uninstalled");

    Ok(())
}
