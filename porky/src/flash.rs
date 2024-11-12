use crate::FLASH_SIZE;
use core::str;
use defmt::info;
use embassy_rp::flash::{Async, Flash};
use embassy_rp::peripherals::{DMA_CH1, FLASH};
use faster_hex::hex_encode;
use static_cell::StaticCell;

/// Get the unique serial number from Flash
pub fn serial_number(flash: FLASH, dma: DMA_CH1) -> &'static str {
    // Get a unique device id - in this case an eight-byte ID from flash rendered as hex string
    let mut flash = Flash::<_, Async, { FLASH_SIZE }>::new(flash, dma);
    let mut device_id = [0; 8];
    flash.blocking_unique_id(&mut device_id).unwrap();
    info!("device_id: {:?}", device_id);

    // convert the device_id to a hex "string"
    let mut device_id_hex: [u8; 16] = [0; 16];
    hex_encode(&device_id, &mut device_id_hex).unwrap();

    info!("device_id: {}", device_id_hex);
    static ID: StaticCell<[u8; 16]> = StaticCell::new();
    let id = ID.init(device_id_hex);
    str::from_utf8(id).unwrap()
}
