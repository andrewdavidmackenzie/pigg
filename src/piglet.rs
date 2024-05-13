mod gpio;

fn main() {
    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);

    let state = gpio::GPIOState::get(&config);

    println!("Oink: {:?}", state);
}