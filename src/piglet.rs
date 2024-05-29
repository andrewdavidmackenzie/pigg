use hw::Hardware;

mod gpio;
#[allow(dead_code)]
mod hw;

fn main() {
    let mut hw = hw::get();
    println!("Pin Descriptions: {:?}", hw.pin_descriptions());
    println!("Hardware: {:?}", hw.descriptor());

    // When we write piglet for real - this will probably be sent over the network from a piggui
    let config = gpio::GPIOConfig::default();
    println!("Pin configs: {:?}", config);
    let _ = hw.apply_config(&config); // TODO handle error
}
