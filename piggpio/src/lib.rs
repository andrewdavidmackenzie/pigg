/// There are two implementations of the `HW` struct.
///
#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
pub mod pi;

use log::info;
#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
pub use pi::HW;
use std::env::current_exe;
use std::fs;
use std::fs::File;
use std::io::Write;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
)))]
pub mod fake_pi;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
)))]
pub use fake_pi::HW;

mod pin_descriptions;

const PIGG_INFO_FILENAME: &str = "pigglet.info";

/// Write a [ListenerInfo] file that captures information that can be used to connect to pigglet
pub fn write_info_file(contents: &str) -> anyhow::Result<()> {
    // remove any leftover file from a previous execution - ignore any failure
    let exec_path = current_exe()?;
    let info_path = exec_path.with_file_name(PIGG_INFO_FILENAME);
    let _ = fs::remove_file(&info_path);

    let mut output = File::create(&info_path)?;
    write!(output, "{contents}")?;
    info!("Info file written at: {info_path:?}");
    Ok(())
}

/*

/// Check that this is the only instance of the process running (user or service)
/// If another version is detected:
/// - print out that fact, with the process ID
/// - print out the nodeid of the instance that is running
/// - exit
#[cfg(any(
    debug_assertions,
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    )
))]
fn check_unique(process_names: &[&str], info_filename: &str) -> anyhow::Result<()> {
    let my_pid = std::process::id();
    let sys = sysinfo::System::new_all();
    for process_name in process_names {
        // Avoid detecting this process instance that is running
        let instances: Vec<&sysinfo::Process> = sys
            .processes_by_exact_name(process_name.as_ref())
            .filter(|p| p.thread_kind().is_none() && p.pid().as_u32() != my_pid)
            .collect();
        if let Some(process) = instances.first() {
            println!(
                "An instance of {process_name} is already running with PID='{}'",
                process.pid(),
            );

            // If we can find the path to the executable - look for the info file
            if let Some(path) = process.exe() {
                let info_path = path.with_file_name(info_filename);
                if info_path.exists() {
                    println!("{}", fs::read_to_string(info_path)?);
                }
            }

            anyhow::bail!("Two instances of {process_name} are not allowed");
        }
    }

    Ok(())
}
 */

/// Get access to GPIO Hardware - making sure we have unique access when we are actually
/// accessing the GPIO hardware on a Pi - creating a file to ensure single access that
/// contains information that maybe useful for other instances trying to gain access also
#[allow(unused_variables)]
pub fn get_hardware(content: &str) -> anyhow::Result<Option<HW>> {
    // release build and Not Pi hardware - No Hardware to get
    #[cfg(all(
        not(debug_assertions),
        not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))
    ))]
    return Ok(None);

    // debug build or Pi Hardware - Return some hardware, fake or real, if we can get
    // exclusive access to it
    #[cfg(any(
        debug_assertions,
        all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        )
    ))]
    {
        //check_unique(&["pigglet"], PIGG_INFO_FILENAME)?;
        //write_info_file(content)?;
        Ok(Some(HW::new(env!("CARGO_PKG_NAME"))))
    }
}

#[cfg(test)]
mod test {
    use pigdef::description::{PinDescription, PinDescriptionSet};
    use pigdef::pin_function::PinFunction;
    use serial_test::serial;
    use std::borrow::Cow;

    #[test]
    fn write_info_file_test() {
        let nodeid = "nodeid: rxci3kuuxljxqej7hau727aaemcjo43zvf2zefnqla4p436sqwhq";
        super::write_info_file(&format!("write_info_file_test\n{nodeid}"))
            .expect("Writing info file failed");
    }

    #[test]
    #[serial] // HW access
    fn get_hardware_test() {
        let hw = crate::get_hardware("get_hardware_test\n")
            .expect("Error getting hardware")
            .expect("Could not get hardware");
        let description = hw.description();
        let pins = description.pins.pins();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }

    #[test]
    #[serial] // HW Access
    fn forty_board_pins_test() {
        let hw = crate::get_hardware("forty_board_pins_test\n")
            .expect("Error getting hardware")
            .expect("Could not get hardware");
        assert_eq!(hw.description().pins.pins().len(), 40);
    }

    #[test]
    #[serial] // HW access
    fn bcm_pins_sort_in_order_test() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = crate::get_hardware("bcm_pins_sort_in_order_test\n")
            .expect("Error getting hardware")
            .expect("Could not get hardware");
        let pin_set = hw.description().pins.clone();
        let sorted_bcm_pins = pin_set.bcm_pins_sorted();
        assert_eq!(pin_set.bcm_pins_sorted().len(), 26);
        let mut previous = 1; // we start at GPIO2
        for pin in sorted_bcm_pins {
            assert_eq!(pin.bcm.expect("Could not get BCM pin number"), previous + 1);
            previous = pin.bcm.expect("Could not get BCM pin number");
        }
    }

    #[test]
    fn display_pin_description() {
        let pin = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![]),
        };

        println!("Pin: {pin}");
    }

    #[test]
    fn sort_bcm() {
        let pin7 = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pin8 = PinDescription {
            bpn: 8,
            bcm: Some(1),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pins = [
            pin7.clone(),
            pin8,
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
        ];
        let pin_set = PinDescriptionSet::new(&pins);
        assert_eq!(
            pin_set
                .pins()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
        assert_eq!(
            pin_set
                .pins()
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
    }
}
