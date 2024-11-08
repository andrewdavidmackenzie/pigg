use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::{Builder, Config};
use static_cell::StaticCell;

pub(crate) fn get_usb_builder(
    driver: Driver<'static, USB>,
    serial: &'static str,
) -> Builder<'static, Driver<'static, USB>> {
    // Create embassy-usb Config
    let mut config = Config::new(0xbabe, 0xface);
    config.manufacturer = Some("pigg");
    config.product = Some("porky");
    config.serial_number = Some(serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 256])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    )
}
