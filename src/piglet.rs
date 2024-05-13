/// When built with the "rppal" feature for interacting with GPIO - can only be built for RPi
#[cfg(feature = "rppal")]
use rppal;

fn main() {
    println!("Oink");
}