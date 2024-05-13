// This binary will only be built when the "rppal" feature for interacting with GPIO is enabled
// so no need for conditional compilation here

mod gpio;

fn main() {
    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);

    let state = gpio::GPIOState::get(&config);

    println!("Oink: {:?}", state);
}