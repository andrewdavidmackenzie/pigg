

/// When built with the "rppal" feature for interacting with GPIO - can only be built for RPi
#[cfg(feature = "rppal")]
use rppal;

/// When built with the "iced" feature for GUI. This can be on Linux, Macos or RPi (linux)
#[cfg(feature = "iced")]
use iced;

fn main() {
    println!("OINK");
}