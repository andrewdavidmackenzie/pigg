use crate::flash;
use crate::flash::DbFlash;
use crate::hw_definition::description::{HardwareDescription, SsidSpec};
use crate::hw_definition::usb_requests::{GET_HARDWARE_VALUE, GET_SSID_VALUE, PIGGUI_REQUEST};
use crate::usb::get_usb_builder;
use core::str;
use defmt::{error, info, unwrap};
use ekv::Database;
use embassy_executor::Spawner;
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::{FLASH, USB};
use embassy_rp::usb::Driver;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::msos::windows_version;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{msos, Handler, UsbDevice};
use static_cell::StaticCell;

type MyDriver = Driver<'static, USB>;

// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    device.run().await
}

/// Handle CONTROL endpoint requests and responses. For many simple requests and responses
/// you can get away with only using the control endpoint.
pub(crate) struct ControlHandler<'h> {
    if_num: InterfaceNumber,
    hardware_description: &'h HardwareDescription<'h>,
    ssid_spec: SsidSpec,
    db: Database<DbFlash<Flash<'h, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    buf: [u8; 1024],
}

/// Update the [SsidSpec] held by the [ControlHandler] and also write it to the flash database
fn store_ssid_spec<'h>(
    _db: &mut Database<DbFlash<Flash<'h, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    _ssid_spec: &SsidSpec,
) {
    // TODO store it
    /*
    let mut wtx = self.db.write_transaction().await;
    let bytes = postcard::to_slice(ssid_spec, &mut self.buf).unwrap();
    wtx.write(SSID_SPEC_KEY, bytes).await.unwrap();
    wtx.commit().await.unwrap();
     */
}

impl<'h> Handler for ControlHandler<'h> {
    /// Respond to HostToDevice control messages, where the host sends us a command and
    /// optionally some data, and we can only acknowledge or reject it.
    fn control_out(&mut self, req: Request, buf: &[u8]) -> Option<OutResponse> {
        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Respond to valid requests from piggui
        match (req.request, req.value) {
            (PIGGUI_REQUEST, _) => {
                self.ssid_spec = postcard::from_bytes::<SsidSpec>(buf).ok()?;
                // TODO store it!
                store_ssid_spec(&mut self.db, &self.ssid_spec);
                info!("SsidSpec set via USB");
            }
            (_, _) => {
                error!(
                    "Unknown USB request and/or value: {}:{}",
                    req.request, req.value
                );
                return Some(OutResponse::Rejected);
            }
        };
        Some(OutResponse::Accepted)
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, _buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Respond to valid requests from piggui
        let msg = match (req.request, req.value) {
            (PIGGUI_REQUEST, GET_HARDWARE_VALUE) => {
                postcard::to_slice(self.hardware_description, &mut self.buf).ok()?
            }
            (PIGGUI_REQUEST, GET_SSID_VALUE) => {
                postcard::to_slice(&self.ssid_spec, &mut self.buf).ok()?
            }
            _ => {
                error!(
                    "Unknown USB request and/or value: {}:{}",
                    req.request, req.value
                );
                return Some(InResponse::Rejected);
            }
        };
        Some(InResponse::Accepted(msg))
    }
}

/// Start the USB stack and raw communications over it
pub async fn start(
    spawner: Spawner,
    driver: Driver<'static, USB>,
    hardware_description: &'static HardwareDescription<'_>,
    db: Database<DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
) {
    let mut builder = get_usb_builder(driver, hardware_description.details.serial);

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature,
    // which causes it to use the built-in WinUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle, you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    static STATIC_BUF: StaticCell<[u8; 200]> = StaticCell::new();
    let static_buf = STATIC_BUF.init([0u8; 200]);
    let ssid_spec = crate::get_ssid_spec(&db, static_buf).await;

    static HANDLER: StaticCell<ControlHandler> = StaticCell::new();
    let handle = HANDLER.init(ControlHandler {
        if_num: InterfaceNumber(0),
        hardware_description,
        ssid_spec,
        db,
        buf: [0; 1024],
    });

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let _alt = interface.alt_setting(0xFF, 0, 0, None);
    handle.if_num = interface.interface_number();
    drop(function);
    builder.handler(handle);

    // Build the builder.
    let usb = builder.build();

    info!("USB raw interface started");

    // Run the USB device.
    unwrap!(spawner.spawn(usb_task(usb)))
}
