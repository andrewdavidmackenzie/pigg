use crate::hw_definition::config::HardwareConfigMessage;
use defmt::info;
#[cfg(feature = "usb-tcp")]
use embassy_futures::select::{select, Either};
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::tcp::{AcceptError, TcpSocket};
use embassy_net::Stack;

pub const TCP_PORT: u16 = 1234;

/// Wait for a TCP connection to be made to this device, then respond to it with the [HardwareDescription]
pub async fn wait_connection<'a>(
    wifi_stack: Stack<'static>,
    #[cfg(feature = "usb-tcp")] usb_stack: Stack<'static>,
    wifi_rx_buffer: &'a mut [u8],
    wifi_tx_buffer: &'a mut [u8],
    #[cfg(feature = "usb-tcp")] usb_rx_buffer: &'a mut [u8],
    #[cfg(feature = "usb-tcp")] usb_tx_buffer: &'a mut [u8],
) -> Result<TcpSocket<'a>, AcceptError> {
    // TODO check these are needed
    let client_state: TcpClientState<2, 1024, 1024> = TcpClientState::new();
    let _client = TcpClient::new(wifi_stack, &client_state);

    accept(
        TcpSocket::new(wifi_stack, wifi_tx_buffer, wifi_rx_buffer),
        #[cfg(feature = "usb-tcp")]
        TcpSocket::new(usb_stack, usb_tx_buffer, usb_rx_buffer),
    )
    .await
}

/// Wait for an incoming TCP connection
async fn accept<'a>(
    mut wifi_socket: TcpSocket<'a>,
    #[cfg(feature = "usb-tcp")] mut usb_socket: TcpSocket<'a>,
) -> Result<TcpSocket<'a>, AcceptError> {
    info!("Listening on port: {}", TCP_PORT);

    #[cfg(feature = "usb-tcp")]
    let socket = match select(wifi_socket.accept(TCP_PORT), usb_socket.accept(TCP_PORT)).await {
        Either::First(s) => match s {
            Ok(_) => wifi_socket,
            Err(e) => return Err(e),
        },
        Either::Second(s) => match s {
            Ok(_) => usb_socket,
            Err(e) => return Err(e),
        },
    };

    #[cfg(not(feature = "usb-tcp"))]
    let socket = match wifi_socket.accept(TCP_PORT).await {
        Ok(_) => wifi_socket,
        Err(e) => return Err(e),
    };

    Ok(socket)
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
