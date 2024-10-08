use crate::ssid::SSID_SECURITY;
use cyw43::Control;
use cyw43::NetDriver;
use cyw43::{JoinAuth, JoinOptions};
use defmt::{error, info, warn};
use embassy_net::Ipv4Address;
use embassy_net::Stack;
use embassy_rp::gpio::Output;
use embassy_time::Timer;

use cyw43_pio::PioSpi;

use embassy_rp::peripherals::{DMA_CH0, PIO0};

const WIFI_JOIN_RETRY_ATTEMPT_LIMIT: usize = 3;

#[embassy_executor::task]
pub async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
pub async fn net_task(stack: &'static Stack<NetDriver<'static>>) -> ! {
    stack.run().await
}

pub async fn join(
    control: &mut Control<'_>,
    stack: &Stack<NetDriver<'static>>,
    ssid_name: &str,
    ssid_pass: &str,
) -> Option<Ipv4Address> {
    let mut attempt = 1;
    while attempt <= WIFI_JOIN_RETRY_ATTEMPT_LIMIT {
        info!(
            "Attempt #{} to join wifi network: '{}' with security = '{}'",
            attempt, ssid_name, SSID_SECURITY
        );

        let mut join_options = JoinOptions::new(ssid_pass.as_bytes());

        match SSID_SECURITY {
            "open" => join_options.auth = JoinAuth::Open,
            "wpa" => join_options.auth = JoinAuth::Wpa,
            "wpa2" => join_options.auth = JoinAuth::Wpa2,
            "wpa3" => join_options.auth = JoinAuth::Wpa3,
            _ => {
                error!("Security '{}' is not supported", SSID_SECURITY);
                return None;
            }
        };

        match control.join(ssid_name, join_options).await {
            Ok(_) => {
                info!("Joined wifi network: '{}'", ssid_name);
                return wait_for_dhcp(stack).await;
            }
            Err(_) => {
                attempt += 1;
                warn!("Failed to join wifi, retrying");
            }
        }
    }

    error!(
        "Failed to join Wifi after {} retries",
        WIFI_JOIN_RETRY_ATTEMPT_LIMIT
    );
    None
}

/// Wait for the DHCP service to come up and for us to get an IP address
async fn wait_for_dhcp(stack: &Stack<NetDriver<'static>>) -> Option<Ipv4Address> {
    info!("Waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");
    if let Some(if_config) = stack.config_v4() {
        Some(if_config.address.address())
    } else {
        None
    }
}

/* Wi-Fi scanning
We could use this to program the ssid config with a list of ssids, and when
it cannot connect via one, it scans to see if another one it knows is available
and then tries to connect to that.

let mut scanner = control.scan(Default::default()).await;
while let Some(bss) = scanner.next().await {
    if let Ok(ssid_str) = str::from_utf8(&bss.ssid) {
    info!("scanned {} == {:x}", ssid_str, bss.bssid);
    }
} */
