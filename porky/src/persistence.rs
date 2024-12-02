use crate::flash;
use crate::flash::DbFlash;
use crate::hw_definition::config::HardwareConfigMessage;
use ekv::Database;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

pub async fn store_config_change<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    hardware_config_message: HardwareConfigMessage,
) -> Result<(), &'static str> {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut wtx = db.write_transaction().await;
    let bytes = postcard::to_slice(&hardware_config_message, &mut buf)
        .map_err(|_| "Deserialization error")?;
    wtx.write(b"pin", bytes).await.map_err(|_| "Write error")?;
    wtx.commit().await.map_err(|_| "Commit error")
}
