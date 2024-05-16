mod gpio;
mod hw;

// Use Hardware via trait
use hw::Hardware;
use crate::gpio::GPIO_DESCRIPTION;

fn main() {
    println!("Description: {:?}", GPIO_DESCRIPTION);

    // When we write piglet for real - this will probably be sent over the network from a piggui
    let config = gpio::GPIOConfig::default();
    println!("Pin configs: {:?}", config);

    let mut hw = hw::get();
    hw.apply_config(&config);
    println!("Oink: {:?}", hw.get_state());
}