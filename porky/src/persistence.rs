use crate::flash::DbFlash;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::description::SsidSpec;
use crate::{flash, ssid};
use defmt::{error, info};
use ekv::{Database, ReadError};
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

/// [SSID_SPEC_KEY] is the key to a possible entry in the Flash DB for SsidSpec override
const SSID_SPEC_KEY: &[u8] = b"ssid_spec";

pub async fn store_config_change<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    hardware_config_message: HardwareConfigMessage,
) -> Result<(), &'static str> {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut wtx = db.write_transaction().await;
    let bytes = postcard::to_slice(&hardware_config_message, &mut buf)
        .map_err(|_| "Deserialization error")?;

    match hardware_config_message {
        NewConfig(config) => {}
        NewPinConfig(bcm, pin_function) => {}
        IOLevelChanged(bcm, level_change) => {}
    }

    wtx.write(b"pin", bytes).await.map_err(|_| "Write error")?;
    wtx.commit().await.map_err(|_| "Commit error")
}

#[cfg(feature = "wifi")]
/// Return an [Option<SsidSpec>] if one could be found in Flash Database or a default.
/// The default, if it exists was built from `ssid.toml` file in project root folder
pub async fn get_ssid_spec<'a>(
    db: &Database<DbFlash<Flash<'a, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    buf: &'a mut [u8],
) -> Option<SsidSpec> {
    let rtx = db.read_transaction().await;
    let spec = match rtx.read(SSID_SPEC_KEY, buf).await {
        Ok(size) => match postcard::from_bytes::<SsidSpec>(&buf[..size]) {
            Ok(spec) => Some(spec),
            Err(_) => {
                error!("Error deserializing SsidSpec from Flash database, trying default");
                ssid::get_default_ssid_spec()
            }
        },
        Err(ReadError::KeyNotFound) => {
            info!("No SsidSpec found in Flash database, trying default");
            ssid::get_default_ssid_spec()
        }
        Err(_) => {
            info!("Error reading SsidSpec from Flash database, trying default");
            ssid::get_default_ssid_spec()
        }
    };

    match &spec {
        None => info!("No SsidSpec used"),
        Some(s) => info!("SsidSpec used for SSID: {}", s.ssid_name),
    }

    spec
}

#[cfg(feature = "wifi")]
/// Write the [SsidSpec] to the flash database
pub async fn store_ssid_spec<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    ssid_spec: SsidSpec,
) -> Result<(), &'static str> {
    let mut buf: [u8; 1024] = [0; 1024];

    let mut wtx = db.write_transaction().await;
    let bytes = postcard::to_slice(&ssid_spec, &mut buf).map_err(|_| "Deserialization error")?;
    wtx.write(SSID_SPEC_KEY, bytes)
        .await
        .map_err(|_| "Write error")?;
    wtx.commit().await.map_err(|_| "Commit error")
}

#[cfg(feature = "wifi")]
/// Delete the [SsidSpec] from the flash database
pub async fn delete_ssid_spec<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
) -> Result<(), &'static str> {
    let mut wtx = db.write_transaction().await;
    wtx.delete(SSID_SPEC_KEY)
        .await
        .map_err(|_| "Delete error")?;
    wtx.commit().await.map_err(|_| "Commit error")
}
