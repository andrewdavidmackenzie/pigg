use crate::flash::DbFlash;
use crate::gpio::Gpio;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
use crate::hw_definition::description::HardwareDescription;
use crate::{flash, persistence, HARDWARE_EVENT_CHANNEL};
use cyw43::Control;
use defmt::info;
use ekv::Database;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_net::tcp::{AcceptError, TcpSocket};
use embassy_net::Stack;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_io_async::Write;

pub const TCP_PORT: u16 = 1234;

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
async fn send_hardware_description_and_config(
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
async fn send_hardware_config(socket: &mut TcpSocket<'_>, hw_config: &HardwareConfig) {
    let mut hw_buf = [0; 1024];
    let slice = postcard::to_slice(hw_config, &mut hw_buf).unwrap();
    info!("Sending hardware config (length: {})", slice.len());
    socket.write_all(slice).await.unwrap()
}

/// Send a [HardwareConfigMessage] over TCP to the GUI
async fn send_message(socket: &mut TcpSocket<'_>, hardware_config_message: HardwareConfigMessage) {
    let mut buf = [0; 1024];
    let gui_message = postcard::to_slice(&hardware_config_message, &mut buf).unwrap();
    socket.write_all(gui_message).await.unwrap();
}

/// Wait until a config message in received on the [TcpSocket] then deserialize it and return it
/// or return `None` if the connection was broken
async fn wait_message(socket: &mut TcpSocket<'_>) -> Option<HardwareConfigMessage> {
    let mut buf = [0; 4096]; // TODO needed?

    let n = socket.read(&mut buf).await.ok()?;
    if n == 0 {
        info!("Connection broken");
        return None;
    }

    postcard::from_bytes(&buf[..n]).ok()
}

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

pub async fn message_loop<'a>(
    gpio: &mut Gpio,
    mut socket: TcpSocket<'_>,
    hw_desc: &HardwareDescription<'_>,
    hw_config: &mut HardwareConfig,
    spawner: &Spawner,
    control: &mut Control<'_>,
    db: &'static Database<
        DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>,
        NoopRawMutex,
    >,
) {
    send_hardware_description_and_config(&mut socket, hw_desc, hw_config).await;

    info!("Entering TCP message loop");
    loop {
        match select(
            wait_message(&mut socket),
            HARDWARE_EVENT_CHANNEL.receiver().receive(),
        )
        .await
        {
            Either::First(config_message) => match config_message {
                None => break,
                Some(hardware_config_message) => {
                    gpio.apply_config_change(control, spawner, &hardware_config_message, hw_config)
                        .await;
                    let _ = persistence::store_config_change(db, &hardware_config_message).await;
                    if matches!(hardware_config_message, HardwareConfigMessage::GetConfig) {
                        send_hardware_config(&mut socket, hw_config).await;
                    }
                    if matches!(hardware_config_message, HardwareConfigMessage::Disconnect) {
                        info!("TCP, Disconnect, exiting TCP Message loop");
                        return;
                    }
                }
            },
            Either::Second(hardware_config_message) => {
                send_message(&mut socket, hardware_config_message.clone()).await;
            }
        }
    }
    info!("Exiting Message Loop");
}
