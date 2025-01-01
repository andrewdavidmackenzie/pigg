use crate::flash;
use crate::flash::DbFlash;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
use crate::hw_definition::description::HardwareDescription;
#[cfg(feature = "wifi")]
use crate::hw_definition::description::{SsidSpec, WiFiDetails};
use crate::hw_definition::usb_values::{
    GET_HARDWARE_DESCRIPTION_VALUE, GET_HARDWARE_DETAILS_VALUE, GET_INITIAL_CONFIG_VALUE,
    PIGGUI_REQUEST, USB_PACKET_SIZE,
};
#[cfg(feature = "wifi")]
use crate::hw_definition::usb_values::{GET_WIFI_VALUE, RESET_SSID_VALUE, SET_SSID_VALUE};
use crate::persistence;
use crate::{gpio, HARDWARE_EVENT_CHANNEL};
use core::str;
#[cfg(feature = "wifi")]
use cyw43::Control;
use defmt::{error, info, unwrap};
use ekv::Database;
use embassy_executor::Spawner;
#[cfg(feature = "wifi")]
use embassy_futures::block_on;
use embassy_futures::select::{select, Either};
use embassy_rp::flash::{Blocking, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Endpoint, In, Out};
#[cfg(feature = "wifi")]
use embassy_rp::watchdog::Watchdog;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Duration;
use embassy_time::Timer;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{EndpointIn, EndpointOut};
use embassy_usb::msos::windows_version;
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{msos, Handler, UsbDevice};
use embassy_usb::{Builder, Config};
use serde::Serialize;
use static_cell::StaticCell;

type MyDriver = Driver<'static, USB>;

// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyDriver>) -> ! {
    info!("USB started");
    device.run().await
}

/// Create a Builder for the USB stack using:
/// vendor: "pigg"
/// product: "porky"
fn get_usb_builder(
    driver: Driver<'static, USB>,
    serial: &'static str,
) -> Builder<'static, Driver<'static, USB>> {
    // Create embassy-usb Config
    let mut config = Config::new(0xbabe, 0xface);
    config.manufacturer = Some("pigg");
    config.product = Some("porky"); // Same for all variants of porky, for Pico, Pico W, Pico 2, Pico 2 W etc.
    config.serial_number = Some(serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;
    config.serial_number = Some(serial);

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

/// Handle CONTROL endpoint requests and responses. For many simple requests and responses
/// you can get away with only using the control endpoint.
pub(crate) struct ControlHandler<'h> {
    if_num: InterfaceNumber,
    hardware_description: &'h HardwareDescription<'h>,
    hardware_config: HardwareConfig,
    #[cfg(feature = "wifi")]
    tcp: Option<([u8; 4], u16)>,
    #[cfg(feature = "wifi")]
    db: &'h Database<DbFlash<Flash<'h, FLASH, Blocking, { flash::FLASH_SIZE }>>, NoopRawMutex>,
    buf: [u8; 1024],
    #[cfg(feature = "wifi")]
    watchdog: Watchdog,
}

impl Handler for ControlHandler<'_> {
    #[allow(unused_variables)] // TODO for now as not used in non-wifi yet
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

        match (req.request, req.value) {
            #[cfg(feature = "wifi")]
            (PIGGUI_REQUEST, SET_SSID_VALUE) => match postcard::from_bytes::<SsidSpec>(buf) {
                Ok(spec) => match block_on(persistence::store_ssid_spec(self.db, spec)) {
                    Ok(_) => {
                        info!("SsidSpec for SSID: stored to Flash Database, restarting in 1sec",);
                        self.watchdog.start(Duration::from_millis(1_000));
                        Some(OutResponse::Accepted)
                    }
                    Err(e) => {
                        error!("Error ({}) storing SsidSpec to Flash Database", e);
                        Some(OutResponse::Rejected)
                    }
                },
                Err(_) => {
                    error!("Could not deserialize SsidSpec sent via USB");
                    Some(OutResponse::Rejected)
                }
            },
            #[cfg(feature = "wifi")]
            (PIGGUI_REQUEST, RESET_SSID_VALUE) => {
                match block_on(persistence::delete_ssid_spec(self.db)) {
                    Ok(_) => {
                        info!("SsidSpec deleted from Flash Database");
                        info!("Restarting in 1sec");
                        self.watchdog.start(Duration::from_millis(1_000));
                        Some(OutResponse::Accepted)
                    }
                    Err(e) => {
                        error!("Error ({}) deleting SsidSpec from Flash Database", e);
                        Some(OutResponse::Rejected)
                    }
                }
            }

            (PIGGUI_REQUEST, HW_CONFIG_MESSAGE) => {
                match postcard::from_bytes::<HardwareConfigMessage>(buf) {
                    Ok(spec) => {
                        info!("Received HardwareConfigMessage over USB");
                        Some(OutResponse::Accepted)
                    }
                    Err(_) => {
                        error!("Could not deserialize HardwareConfigMessage sent via USB");
                        Some(OutResponse::Rejected)
                    }
                }
            }

            (_, _) => {
                error!(
                    "Unknown USB request and/or value: {}:{}",
                    req.request, req.value
                );
                Some(OutResponse::Rejected)
            }
        }
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
            (PIGGUI_REQUEST, GET_HARDWARE_DESCRIPTION_VALUE) => {
                postcard::to_slice(self.hardware_description, &mut self.buf).ok()?
            }
            (PIGGUI_REQUEST, GET_HARDWARE_DETAILS_VALUE) => {
                postcard::to_slice(&self.hardware_description.details, &mut self.buf).ok()?
            }
            #[cfg(feature = "wifi")]
            (PIGGUI_REQUEST, GET_WIFI_VALUE) => unsafe {
                static mut STATIC_BUF: [u8; 200] = [0u8; 200];
                #[allow(static_mut_refs)]
                let ssid_spec = block_on(persistence::get_ssid_spec(self.db, &mut STATIC_BUF));
                let wifi = WiFiDetails {
                    ssid_spec,
                    tcp: self.tcp,
                };
                postcard::to_slice(&wifi, &mut self.buf).ok()?
            },
            (PIGGUI_REQUEST, GET_INITIAL_CONFIG_VALUE) => {
                let slice = postcard::to_slice(&self.hardware_config, &mut self.buf).ok()?;
                info!(
                    "Returning Initial hardware config by USB: size = {}",
                    slice.len()
                );
                slice
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

/// [UsbConnection] is used to send and receive messages back and forth to the host, but not using
/// the Control transfers, instead using InterruptIn or InterruptOut transfers
pub struct UsbConnection<D: EndpointIn, E: EndpointOut> {
    ep_in: D,
    ep_out: E,
    buf: &'static mut [u8; 1024],
}

impl<D: EndpointIn, E: EndpointOut> UsbConnection<D, E> {
    /// Take the [HardwareConfigMessage] serialize it using postcard and send it to the host
    /// via the [EndpointIn]
    pub async fn send(&mut self, msg: impl Serialize) {
        let msg = postcard::to_slice(&msg, self.buf).unwrap(); // TODO
        info!("USB Send {} bytes", msg.len());
        self.ep_in.write(msg).await.unwrap(); // TODO
    }

    /// Receive a [HardwareConfigMessage] from the host over the usb Endpoint out
    pub async fn receive(&mut self) -> HardwareConfigMessage {
        let delay = Duration::from_secs(1);
        loop {
            let size = self.ep_out.read(self.buf).await.unwrap(); // TODO
            if size != 0 {
                info!("USB Receive: {} bytes", size);
                postcard::from_bytes(&self.buf[..size]).unwrap()
            } else {
                Timer::after(delay).await;
            }
        }
    }
}

/// Start the USB stack and raw communications over it
// <D: embassy_usb_driver::Driver<'static>>
pub async fn start(
    spawner: Spawner,
    driver: Driver<'static, USB>,
    hardware_description: &'static HardwareDescription<'_>,
    hardware_config: HardwareConfig,
    #[cfg(feature = "wifi")] tcp: Option<([u8; 4], u16)>,
    #[cfg(feature = "wifi")] db: &'static Database<
        DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>,
        NoopRawMutex,
    >,
    #[cfg(feature = "wifi")] watchdog: Watchdog,
) -> UsbConnection<Endpoint<'static, USB, In>, Endpoint<'static, USB, Out>> {
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

    static CONTROL_HANDLER: StaticCell<ControlHandler> = StaticCell::new();
    let control_handler = CONTROL_HANDLER.init(ControlHandler {
        if_num: InterfaceNumber(0),
        hardware_description,
        hardware_config,
        #[cfg(feature = "wifi")]
        tcp,
        #[cfg(feature = "wifi")]
        db,
        buf: [0; 1024],
        #[cfg(feature = "wifi")]
        watchdog,
    });

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let _alt = interface.alt_setting(0xFF, 0, 0, None);
    control_handler.if_num = interface.interface_number();

    let mut alt = interface.alt_setting(0xFF, 0, 0, None);
    let ep_in = alt.endpoint_interrupt_in(USB_PACKET_SIZE, 10);
    let ep_out = alt.endpoint_interrupt_out(USB_PACKET_SIZE, 10);

    drop(function);
    builder.handler(control_handler);

    let usb = builder.build();

    unwrap!(spawner.spawn(usb_task(usb)));

    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let buf = BUF.init([0u8; 1024]);
    UsbConnection { ep_in, ep_out, buf }
}

pub async fn message_loop(
    mut usb_connection: UsbConnection<Endpoint<'static, USB, In>, Endpoint<'static, USB, Out>>,
    _hw_desc: &HardwareDescription<'_>,
    hw_config: &mut HardwareConfig,
    spawner: &Spawner,
    #[cfg(feature = "wifi")] control: &mut Control<'_>,
    db: &'static Database<
        DbFlash<Flash<'static, FLASH, Blocking, { flash::FLASH_SIZE }>>,
        NoopRawMutex,
    >,
) {
    // TODO Wait for message requesting hardware_description and config that initiates a 'connection'

    info!("Entering USB message loop");
    loop {
        match select(
            usb_connection.receive(),
            HARDWARE_EVENT_CHANNEL.receiver().receive(),
        )
        .await
        {
            Either::First(hardware_config_message) => {
                gpio::apply_config_change(
                    #[cfg(feature = "wifi")]
                    control,
                    spawner,
                    &hardware_config_message,
                    hw_config,
                )
                .await;
                let _ = persistence::store_config_change(db, &hardware_config_message).await;
                if matches!(hardware_config_message, HardwareConfigMessage::GetConfig) {
                    usb_connection.send(&hw_config).await;
                }
            }
            Either::Second(hardware_config_message) => {
                usb_connection.send(hardware_config_message).await;
            }
        }
    }
    //info!("Exiting Message Loop");
}
