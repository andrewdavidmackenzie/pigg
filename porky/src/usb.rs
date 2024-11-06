use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_rp::peripherals;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_ncm::embassy_net::{Device, Runner, State as NetState};
use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use rand::RngCore;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

type MyDriver = Driver<'static, peripherals::USB>;

const MTU: usize = 1514;

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn usb_ncm_task(class: Runner<'static, MyDriver, MTU>) -> ! {
    class.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static, MTU>>) -> ! {
    runner.run().await
}

pub async fn start_net(
    spawner: Spawner,
    driver: Driver<'static, USB>,
    serial: &'static str,
) -> Stack<'static> {
    let mut rng = RoscRng;

    // Create embassy-usb Config
    let mut config = Config::new(0xbabe, 0xface);
    config.manufacturer = Some("pigg");
    config.product = Some("porky");
    config.serial_number = Some(serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut [], // no msos descriptors
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Our MAC addr.
    let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
    let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

    // Create classes on the builder.
    static STATE: StaticCell<State> = StaticCell::new();
    let class = CdcNcmClass::new(&mut builder, STATE.init(State::new()), host_mac_addr, 64);
    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));

    static NET_STATE: StaticCell<NetState<MTU, 4, 4>> = StaticCell::new();
    let (runner, device) =
        class.into_embassy_net_device::<MTU, 4, 4>(NET_STATE.init(NetState::new()), our_mac_addr);
    unwrap!(spawner.spawn(usb_ncm_task(runner)));

    let config = embassy_net::Config::dhcpv4(Default::default());
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    //});

    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    unwrap!(spawner.spawn(net_task(runner)));

    //    wifi::wait_for_dhcp("USB", &stack).await;

    stack
}
