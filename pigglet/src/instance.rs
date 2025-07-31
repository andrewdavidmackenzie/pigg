#[cfg(feature = "iroh")]
use crate::device_net::iroh_device;
#[cfg(feature = "tcp")]
use crate::device_net::tcp_device;
use anyhow::{anyhow, Context};
use log::info;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The [InstanceInfo] struct captures information about network connections the instance of
/// `pigglet` is listening on, that can be used with `piggui` to start a remote GPIO session
pub(crate) struct InstanceInfo {
    pub(crate) process_name: String,
    pub(crate) pid: u32,
    #[cfg(feature = "iroh")]
    pub(crate) iroh_info: iroh_device::IrohDevice,
    #[cfg(feature = "tcp")]
    pub(crate) tcp_info: tcp_device::TcpDevice,
}

impl InstanceInfo {
    pub(crate) fn load_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let string = fs::read_to_string(path)?;
        let mut lines = string.lines();
        let process_name = lines
            .next()
            .ok_or_else(|| anyhow!("Missing process name"))?
            .to_string();
        let pid = lines
            .next()
            .ok_or_else(|| anyhow!("Missing PID"))?
            .parse::<u32>()
            .context("Invalid PID")?;

        #[cfg(feature = "iroh")]
        let iroh_info = iroh_device::IrohDevice::parse(&mut lines)?;

        #[cfg(feature = "tcp")]
        let tcp_info = tcp_device::TcpDevice::parse(&mut lines)?;

        Ok(Self {
            process_name,
            pid,
            #[cfg(feature = "iroh")]
            iroh_info,
            #[cfg(feature = "tcp")]
            tcp_info,
        })
    }

    /// Write a [InstanceInfo] file that captures information that can be used to connect to pigglet
    pub(crate) fn write_to_file(&self, info_path: &Path) -> anyhow::Result<()> {
        let mut output = File::create(info_path)?;
        write!(output, "{self}")?;
        info!("Info file written at: {info_path:?}");
        Ok(())
    }
}

impl std::fmt::Display for InstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.process_name)?;
        writeln!(f, "{}", self.pid)?;
        #[cfg(feature = "iroh")]
        write!(f, "{}", self.iroh_info)?;

        #[cfg(feature = "tcp")]
        write!(f, "{}", self.tcp_info)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::InstanceInfo;
    use iroh::{NodeId, RelayUrl};
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::{fs, process};
    use tempfile::tempdir;

    fn listener_info(nodeid: &NodeId, relay_url_str: &str) -> InstanceInfo {
        InstanceInfo {
            process_name: "pigglet_tests".to_string(),
            pid: process::id(),
            #[cfg(feature = "iroh")]
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

    #[test]
    fn write_info_file() {
        let output_dir = tempdir().expect("Could not create a tempdir").keep();
        let test_file = output_dir.join("test.info");
        let nodeid = iroh::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
            .expect("Could not create nodeid");
        let listener_info = listener_info(&nodeid, "https://euw1-1.relay.iroh.network./ ");
        listener_info
            .write_to_file(&test_file)
            .expect("Writing info file failed");
        assert!(test_file.exists(), "File was not created as expected");
        let pigglet_info = fs::read_to_string(test_file).expect("Could not read info file");
        println!("pigglet_info: {pigglet_info}");
        assert!(pigglet_info.contains(&nodeid.to_string()))
    }

    #[test]
    fn write_info_file_non_existent() {
        let output_dir = PathBuf::from("/foo");
        let test_file = output_dir.join("test.info");
        let nodeid = iroh::NodeId::from_str("rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq")
            .expect("Could not create nodeid");
        let listener_info = listener_info(&nodeid, "https://euw1-1.relay.iroh.network./ ");
        assert!(listener_info.write_to_file(&test_file).is_err());
        assert!(!test_file.exists(), "File was created!");
    }
}
