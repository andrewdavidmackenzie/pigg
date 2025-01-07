use crate::flash;
use crate::flash::DbFlash;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::{
    GetConfig, IOLevelChanged, NewConfig, NewPinConfig,
};
#[cfg(feature = "wifi")]
use crate::hw_definition::description::SsidSpec;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::pin_function::PinFunction::Output;
use crate::hw_definition::BCMPinNumber;
#[cfg(feature = "wifi")]
use crate::ssid;
#[cfg(feature = "wifi")]
use defmt::error;
use defmt::info;
use ekv::Database;
#[cfg(feature = "wifi")]
use ekv::ReadError;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use heapless::FnvIndexMap;

#[cfg(feature = "wifi")]
/// [SSID_SPEC_KEY] is the key to a possible entry in the Flash DB for SsidSpec override
const SSID_SPEC_KEY: &[u8] = b"ssid_spec";

/// Load any pre-existing config from flash, if there is none then just return a default config
pub async fn get_config<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
) -> HardwareConfig {
    let mut pin_functions: FnvIndexMap<BCMPinNumber, PinFunction, 32> = FnvIndexMap::new();
    let mut buf: [u8; 1024] = [0; 1024];
    let wtx = db.read_transaction().await;

    for pin_number in 0u8..32u8 {
        if let Ok(size) = wtx.read(&[pin_number], &mut buf).await {
            info!("Found config in flash for pin: {}", pin_number);
            if let Ok(pin_function) = postcard::from_bytes::<PinFunction>(&buf[..size]) {
                let pin_no = pin_number as BCMPinNumber;
                let _ = pin_functions.insert(pin_no, pin_function);
            }
        }
    }

    match pin_functions.is_empty() {
        true => info!("No Config found in flash, starting with default config"),
        false => info!(
            "Config retreived from Flash DB for {} pins",
            pin_functions.len()
        ),
    }

    HardwareConfig { pin_functions }
}

pub async fn store_config_change<'p>(
    db: &Database<DbFlash<Flash<'p, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    hardware_config_message: &HardwareConfigMessage,
) -> Result<(), &'static str> {
    let mut buf: [u8; 1024] = [0; 1024];
    let mut wtx = db.write_transaction().await;

    match hardware_config_message {
        NewConfig(config) => {
            // Delete all previous pin configs
            for pin_number in 0u8..32u8 {
                let _ = wtx.delete(&[pin_number]).await;
            }
            wtx.commit().await.map_err(|_| "Commit error")?;

            // Need to do deletes and writes in separate transactions - with keys in ascending order
            wtx = db.write_transaction().await;
            // Write the new pin configs for all pins in the config
            for (bcm, pin_function) in &config.pin_functions {
                let bytes = postcard::to_slice(&pin_function, &mut buf)
                    .map_err(|_| "Deserialization error")?;
                wtx.write(&[*bcm], bytes).await.map_err(|_| "Write Error")?;
            }
        }
        NewPinConfig(bcm, pin_function) => {
            // Write the new pin config, replacing an old one if it exists
            let bytes =
                postcard::to_slice(&pin_function, &mut buf).map_err(|_| "Deserialization error")?;
            wtx.write(&[*bcm], bytes).await.map_err(|_| "Write Error")?;
        }
        IOLevelChanged(bcm, level_change) => {
            // Write the new pin config (including the new output level), replacing any old one
            let pin_function: PinFunction = Output(Some(level_change.new_level));
            let bytes =
                postcard::to_slice(&pin_function, &mut buf).map_err(|_| "Deserialization error")?;
            wtx.write(&[*bcm], bytes).await.map_err(|_| "Write Error")?;
        }
        GetConfig => { /* Nothing to do in persistence */ }
    }

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
    match rtx.read(SSID_SPEC_KEY, buf).await {
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
    }
}

#[allow(dead_code)] // Not used in porky
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

#[allow(dead_code)] // Not used in porky
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
