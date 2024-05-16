mod gpio;
mod hw;

// Use Hardware via trait
use hw::Hardware;
use crate::gpio::GPIO_DESCRIPTION;

fn main() {
    println!("Description: {:?}", GPIO_DESCRIPTION);

    let config = gpio::GPIOConfig::default();
    println!("Pin configs: {:?}", config);

    let mut hw = hw::get();
    hw.apply_config(&config);
    println!("Oink: {:?}", hw.get_state());
}