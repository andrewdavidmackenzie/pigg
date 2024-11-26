use crate::hw_definition::config::HardwareConfigMessage;
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

    accept(TcpSocket::new(wifi_stack, wifi_tx_buffer, wifi_rx_buffer)).await
}

/// Wait for an incoming TCP connection
async fn accept<'a>(mut wifi_socket: TcpSocket<'a>) -> Result<TcpSocket<'a>, AcceptError> {
    info!("Listening on port: {}", TCP_PORT);

    let socket = match wifi_socket.accept(TCP_PORT).await {
        Ok(_) => wifi_socket,
        Err(e) => return Err(e),
    };

    Ok(socket)
}

/// Send the [HardwareDescription] over the [TcpSocket]
pub async fn send_hardware_description(
    socket: &mut TcpSocket<'_>,
    hw_desc: &HardwareDescription<'_>,
) {
    let mut hw_buf = [0; 1024];
    let slice = postcard::to_slice(hw_desc, &mut hw_buf).unwrap();
    info!("Sending hardware description (length: {})", slice.len());
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
