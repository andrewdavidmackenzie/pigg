use clap::ArgMatches;
use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use std::io;
use std::path::Path;
use std::process::exit;
use std::time::Duration;

const SERVICE_NAME: &str = "net.mackenzie-serres.pigg.pigglet";

/// Handle any service installation or uninstallation tasks specified on the command line
/// continue without doing anything if none were specified
pub(crate) fn manage(exec_path: &Path, matches: &ArgMatches) -> anyhow::Result<()> {
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
    // Run from the dir where exec is for now, so it should find the config file in the ancestor's path
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
fn uninstall_service(service_name: &ServiceLabel) -> Result<(), io::Error> {
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
