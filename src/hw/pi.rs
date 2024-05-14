/// Implementation of GPIO for raspberry pi - uses rrpal
#[cfg(feature = "rppal")]
use rppal::gpio::{InputPin, Level, Trigger};

pub struct GPIOHW;

impl GPIO for GPIOHW {

}