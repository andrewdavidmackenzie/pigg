mod gpio;
mod hw;

// Use Hardware via trait
use hw::Hardware;

fn main() {
    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);
    println!("Pin1 Config is: {:?}", config.pins[1]);

    let mut hw = hw::get();
    hw.apply_config(&config);
    println!("Oink: {:?}", hw.get_state());
}