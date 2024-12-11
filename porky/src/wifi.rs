use crate::hw_definition::description::SsidSpec;
use core::net::Ipv4Addr;
use cyw43::Control;
use cyw43::{JoinAuth, JoinOptions};
use cyw43_pio::PioSpi;
use defmt::{error, info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_net::{Config, Runner, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::Level;
use embassy_rp::gpio::Output;
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_time::Timer;
use rand::RngCore;
use static_cell::StaticCell;

const WIFI_JOIN_RETRY_ATTEMPT_LIMIT: usize = 3;

pub const STACK_RESOURCES_SOCKET_COUNT: usize = 5;

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

pub async fn join(
    control: &mut Control<'_>,
    stack: Stack<'static>,
    wifi_spec: &SsidSpec,
) -> Result<Ipv4Addr, &'static str> {
    let mut attempt = 1;
    while attempt <= WIFI_JOIN_RETRY_ATTEMPT_LIMIT {
        info!(
            "Attempt #{} to join wifi network: '{}' with security = '{}'",
            attempt, wifi_spec.ssid_name, wifi_spec.ssid_security
        );

        let mut join_options = JoinOptions::new(wifi_spec.ssid_pass.as_bytes());

        match wifi_spec.ssid_security.as_str() {
            "open" => join_options.auth = JoinAuth::Open,
            "wpa" => join_options.auth = JoinAuth::Wpa,
            "wpa2" => join_options.auth = JoinAuth::Wpa2,
            "wpa3" => join_options.auth = JoinAuth::Wpa3,
            _ => {
                error!("Security '{}' is not supported", wifi_spec.ssid_security);
                return Err("Security of SsidSpec i snot supported");
            }
        };

        match control.join(&wifi_spec.ssid_name, join_options).await {
            Ok(_) => {
                info!("Joined Wi-Fi network: '{}'", wifi_spec.ssid_name);
                return wait_for_ip(&stack).await;
            }
            Err(_) => {
                attempt += 1;
                warn!("Failed to join wifi, retrying");
            }
        }
    }

    Err("Failed to join Wifi after too many retries")
}

/// Wait for the DHCP service to come up and for us to get an IP address
async fn wait_for_ip(stack: &Stack<'static>) -> Result<Ipv4Addr, &'static str> {
    info!("Waiting for an IP address");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    let if_config = stack.config_v4().ok_or("Could not get IP Config")?;
    Ok(if_config.address.address())
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

/// Initialize the cyw43 chip and start device_net
pub async fn start_net<'a>(
    spawner: Spawner,
    pin_23: embassy_rp::peripherals::PIN_23,
    spi: PioSpi<'static, PIO0, 0, DMA_CH0>,
) -> (Control<'a>, Stack<'static>) {
    let fw = include_bytes!("../assets/43439A0.bin");
    let clm = include_bytes!("../assets/43439A0_clm.bin");
    let pwr = Output::new(pin_23, Level::Low);

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    spawner.spawn(wifi_task(runner)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    static RESOURCES: StaticCell<StackResources<STACK_RESOURCES_SOCKET_COUNT>> = StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    let config = Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Init network stack
    let (stack, runner) = embassy_net::new(net_device, config, resources, seed);

    unwrap!(spawner.spawn(net_task(runner)));

    (control, stack)
}
