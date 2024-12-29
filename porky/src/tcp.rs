use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
use crate::hw_definition::description::HardwareDescription;
use defmt::info;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::tcp::{AcceptError, TcpSocket};
use embassy_net::Stack;
use embedded_io_async::Write;

pub const TCP_PORT: u16 = 1234;

/// Wait for a TCP connection to be made to this device, then respond to it with the [HardwareDescription]
pub async fn wait_connection<'a>(
    wifi_stack: Stack<'static>,
    wifi_rx_buffer: &'a mut [u8],
    wifi_tx_buffer: &'a mut [u8],
) -> Result<TcpSocket<'a>, AcceptError> {
    // TODO check these are needed
    let client_state: TcpClientState<2, 1024, 1024> = TcpClientState::new();
    let _client = TcpClient::new(wifi_stack, &client_state);
    let tcp_socket = TcpSocket::new(wifi_stack, wifi_tx_buffer, wifi_rx_buffer);
    accept(tcp_socket).await
}

/// Wait for an incoming TCP connection
async fn accept(mut wifi_socket: TcpSocket<'_>) -> Result<TcpSocket<'_>, AcceptError> {
    info!("Listening for TCP Connection on port: {}", TCP_PORT);

    let socket = match wifi_socket.accept(TCP_PORT).await {
        Ok(_) => wifi_socket,
        Err(e) => return Err(e),
    };

    Ok(socket)
}

/// Send the [HardwareDescription] and [HardwareConfig] over the [TcpSocket]
pub async fn send_hardware_description_and_config(
    socket: &mut TcpSocket<'_>,
    hw_desc: &HardwareDescription<'_>,
    hw_config: &HardwareConfig,
) {
    let mut hw_buf = [0; 2048];
    let slice = postcard::to_slice(&(hw_desc, hw_config), &mut hw_buf).unwrap();
    info!("Sending hardware description (length: {})", slice.len());
    socket.write_all(slice).await.unwrap()
}

/// Send the [HardwareConfig] over the [TcpSocket]
pub async fn send_hardware_config(socket: &mut TcpSocket<'_>, hw_config: &HardwareConfig) {
    let mut hw_buf = [0; 1024];
    let slice = postcard::to_slice(hw_config, &mut hw_buf).unwrap();
    info!("Sending hardware config (length: {})", slice.len());
    socket.write_all(slice).await.unwrap()
}

/// Send a [HardwareConfigMessage] over TCP to the GUI
pub async fn send_message(
    socket: &mut TcpSocket<'_>,
    hardware_config_message: HardwareConfigMessage,
) {
    let mut buf = [0; 1024];
    let gui_message = postcard::to_slice(&hardware_config_message, &mut buf).unwrap();
    socket.write_all(gui_message).await.unwrap();
}

/// Wait until a config message in received on the [TcpSocket] then deserialize it and return it
/// or return `None` if the connection was broken
pub async fn wait_message(socket: &mut TcpSocket<'_>) -> Option<HardwareConfigMessage> {
    let mut buf = [0; 4096]; // TODO needed?

    let n = socket.read(&mut buf).await.ok()?;
    if n == 0 {
        info!("Connection broken");
        return None;
    }

    postcard::from_bytes(&buf[..n]).ok()
}
