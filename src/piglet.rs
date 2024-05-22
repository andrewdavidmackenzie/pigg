use hw::Hardware;

// Use Hardware via trait
use crate::gpio::GPIO_DESCRIPTION;

mod gpio;
mod hw;

fn main() {
    println!("Description: {:?}", GPIO_DESCRIPTION);

    // When we write piglet for real - this will probably be sent over the network from a piggui
    let config = gpio::GPIOConfig::default();
    println!("Pin configs: {:?}", config);

    let mut hw = hw::get();
    let _ = hw.apply_config(&config); // TODO handle error
    println!("Oink: {:?}", hw.get_state());
}
