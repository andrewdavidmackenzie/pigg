#[cfg(feature = "pico1")]
use core::str;
use defmt::{error, info};
use ekv::flash::{self, PageID};
use ekv::{config, Database};
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
#[cfg(feature = "pico1")]
use faster_hex::hex_encode;
#[cfg(feature = "pico1")]
use static_cell::StaticCell;

#[cfg(not(any(feature = "pico1", feature = "pico2")))]
compile_error!("You must choose either feature \"pico1\" or \"pico2\"");

#[cfg(all(feature = "pico1", not(feature = "pico2")))]
pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
#[cfg(all(feature = "pico2", not(feature = "pico1")))]
pub const FLASH_SIZE: usize = 4 * 1024 * 1024;

extern "C" {
    // Flash storage used for configuration
    static __config_start: u32;
}

// Workaround for alignment requirements.
#[repr(C, align(4))]
struct AlignedBuf<const N: usize>([u8; N]);

pub struct DbFlash<T: NorFlash + ReadNorFlash> {
    start: usize,
    flash: T,
}

#[cfg(feature = "pico1")]
/// Get the unique serial number from Flash
pub fn serial_number(
    flash: &mut Flash<'_, FLASH, Blocking, FLASH_SIZE>,
) -> Result<&'static str, &'static str> {
    // Get a unique device id - in this case an eight-byte ID from flash rendered as hex string
    let mut device_id = [0; 8];
    flash
        .blocking_unique_id(&mut device_id)
        .map_err(|_| "Could not get unique ID from flash")?;

    // convert the device_id to a hex "string"
    let mut device_id_hex: [u8; 16] = [0; 16];
    hex_encode(&device_id, &mut device_id_hex).map_err(|_| "Could not convert unique ID to HEX")?;

    static ID: StaticCell<[u8; 16]> = StaticCell::new();
    let id = ID.init(device_id_hex);
    let device_id_str =
        str::from_utf8(id).map_err(|_| "Could not create String of HEX unique ID")?;
    info!("device_id: {}", device_id_str);
    Ok(device_id_str)
}

impl<T: NorFlash + ReadNorFlash> flash::Flash for DbFlash<T> {
    type Error = T::Error;

    fn page_count(&self) -> usize {
        config::MAX_PAGE_COUNT
    }

    async fn erase(&mut self, page_id: PageID) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        self.flash.erase(
            (self.start + page_id.index() * config::PAGE_SIZE) as u32,
            (self.start + page_id.index() * config::PAGE_SIZE + config::PAGE_SIZE) as u32,
        )
    }

    async fn read(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &mut [u8],
    ) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        let address = self.start + page_id.index() * config::PAGE_SIZE + offset;
        let mut buf = AlignedBuf([0; config::PAGE_SIZE]);
        self.flash.read(address as u32, &mut buf.0[..data.len()])?;
        data.copy_from_slice(&buf.0[..data.len()]);
        Ok(())
    }

    async fn write(
        &mut self,
        page_id: PageID,
        offset: usize,
        data: &[u8],
    ) -> Result<(), <DbFlash<T> as flash::Flash>::Error> {
        let address = self.start + page_id.index() * config::PAGE_SIZE + offset;
        let mut buf = AlignedBuf([0; config::PAGE_SIZE]);
        buf.0[..data.len()].copy_from_slice(data);
        self.flash.write(address as u32, &buf.0[..data.len()])
    }
}

pub fn get_flash<'a>(flash_pin: FLASH) -> Flash<'a, FLASH, Blocking, FLASH_SIZE> {
    Flash::new_blocking(flash_pin)
}

pub async fn db_init(
    flash: Flash<'static, FLASH, Blocking, FLASH_SIZE>,
) -> Database<DbFlash<Flash<'static, FLASH, Blocking, FLASH_SIZE>>, NoopRawMutex> {
    #[cfg(feature = "pico1")]
    const OFFSET: usize = 0x0;
    #[cfg(not(feature = "pico1"))] // The start of flash address needs adjusting on pico 2
    const OFFSET: usize = 0x1000_0000;

    let flash: DbFlash<Flash<_, _, FLASH_SIZE>> = DbFlash {
        flash,
        start: unsafe { &__config_start as *const u32 as usize - OFFSET },
    };
    let db = Database::<_, NoopRawMutex>::new(flash, ekv::Config::default());

    if db.mount().await.is_err() {
        info!("Formatting Flash DB...");
        match db.format().await {
            Ok(..) => info!("Flash Database is up"),
            Err(_e) => error!("Error initializing Flash DB"),
        }
    }

    db
}
