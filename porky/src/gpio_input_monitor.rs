use crate::hw_definition::config::HardwareConfigMessage::IOLevelChanged;
use crate::hw_definition::config::LevelChange;
use crate::hw_definition::BCMPinNumber;
use crate::HARDWARE_EVENT_CHANNEL;
use defmt::info;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{Flex, Level};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::Instant;

/// Wait until a level change on an input occurs and then send it via TCP to GUI
#[embassy_executor::task(pool_size = 32)]
pub async fn monitor_input(
    bcm_pin_number: BCMPinNumber,
    signaller: Receiver<'static, ThreadModeRawMutex, bool, 1>,
    returner: Sender<'static, ThreadModeRawMutex, Flex<'static>, 1>,
    mut flex: Flex<'static>,
) {
    let mut level = flex.get_level();
    send_input_level(bcm_pin_number, level).await;

    loop {
        match select(flex.wait_for_any_edge(), signaller.receive()).await {
            Either::First(()) => {
                let new_level = flex.get_level();
                if new_level != level {
                    send_input_level(bcm_pin_number, flex.get_level()).await;
                    level = new_level;
                }
            }
            Either::Second(_) => {
                debug!("Input Monitor returning Pin");
                let _ = returner.send(flex).await;
                break;
            }
        }
    }
}

/// Send a detected input level change back to the GUI, timestamping with the Duration since boot
async fn send_input_level(bcm: BCMPinNumber, level: Level) {
    let level_change = LevelChange::new(
        level == Level::High,
        Instant::now().duration_since(Instant::MIN).into(),
    );
    let hardware_event = IOLevelChanged(bcm, level_change);
    HARDWARE_EVENT_CHANNEL.sender().send(hardware_event).await;
}
