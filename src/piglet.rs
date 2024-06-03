use hw::Hardware;

mod gpio;
#[allow(dead_code)]
mod hw;

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
fn main() {
    let mut hw = hw::get();
    println!("{}", hw.descriptor().unwrap());
    println!("Pin Descriptions: {:?}", hw.pin_descriptions());

    // When we write piglet for real - this will probably be sent over the network from a piggui
    let config = gpio::GPIOConfig::default();
    println!("Pin configs: {:?}", config);
    let _ = hw.apply_config(&config, |_, _| {}); // TODO handle error
}
