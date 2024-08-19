use crate::hw::Hardware;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub(crate) struct TcpInfo {
    pub ip: String,
    pub port: u16,
}

impl Display for TcpInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "IP Address: {}", self.ip)?;
        writeln!(f, "Port: {}", self.port)?;
        Ok(())
    }
}

pub(crate) async fn get_tcp_listener_info() -> anyhow::Result<TcpInfo> {
    Ok(TcpInfo {
        ip: "10.0.0.1".to_string(),
        port: 9001,
    })
}

pub(crate) async fn listen_tcp(
    _tcp_info: TcpInfo,
    _hardware: &mut impl Hardware,
) -> anyhow::Result<()> {
    Ok(())
}
