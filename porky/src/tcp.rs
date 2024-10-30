use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};
use crate::pin_descriptions::PIN_DESCRIPTIONS;
use core::str::from_utf8;
use cyw43::NetDriver;
use defmt::{error, info};
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::tcp::TcpSocket;
use embassy_net::Ipv4Address;
use embassy_net::Stack;
use embedded_io_async::Write;
use faster_hex::hex_encode;
use heapless::Vec;

/// Wait for a TCP connection to be made to this device
pub async fn wait_connection<'a>(
    stack: &'a Stack<NetDriver<'static>>,
    device_id: [u8; 8],
    ip_address: Ipv4Address,
    rx_buffer: &'a mut [u8],
    tx_buffer: &'a mut [u8],
) -> TcpSocket<'a> {
    // TODO check these are needed
    let client_state: TcpClientState<2, 1024, 1024> = TcpClientState::new();
    let _client = TcpClient::new(stack, &client_state);

    let mut socket = TcpSocket::new(stack, tx_buffer, rx_buffer);
    //socket.set_timeout(Some(Duration::from_secs(10)));

    // wait for an incoming TCP connection
    accept(&mut socket, &ip_address, &device_id).await;

    socket
}

/// Wait for an incoming TCP connection, then respond to it with the [HardwareDescription]
async fn accept(socket: &mut TcpSocket<'_>, ip_address: &Ipv4Address, device_id: &[u8; 8]) {
    let mut buf = [0; 4096];

    info!("Listening on TCP {}:1234", ip_address);
    if let Err(e) = socket.accept(1234).await {
        error!("TCP accept error: {:?}", e);
        return;
    }

    info!(
        "Received connection from {:?}",
        socket.remote_endpoint().unwrap()
    );

    let mut device_id_hex: [u8; 16] = [0; 16];
    hex_encode(device_id, &mut device_id_hex).unwrap();

    // send hardware description
    let details = HardwareDetails {
        model: "Pi Pico W",
        hardware: "RP2040",
        revision: "",
        serial: from_utf8(&device_id_hex).unwrap(),
    };

    let hw_desc = HardwareDescription {
        details,
        pins: PinDescriptionSet {
            pins: Vec::from_slice(&PIN_DESCRIPTIONS).unwrap(),
        },
    };

    let slice = postcard::to_slice(&hw_desc, &mut buf).unwrap();
    info!("Sending hardware description (length: {})", slice.len());
    socket.write_all(slice).await.unwrap();
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
