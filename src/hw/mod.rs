use crate::gpio::{GPIOConfig, GPIOState};

#[cfg_attr(all(feature = "pico", not(feature = "pi")), path = "pico.rs")]
#[cfg_attr(all(feature = "pi", not(feature = "pico")), path = "pi.rs")]
#[cfg_attr(not(any(feature = "pico", feature = "pi")), path = "none.rs")]
mod implementation;

#[allow(dead_code)] // TODO remove later when used
pub fn get() -> impl Hardware {
    implementation::get()
}

// TODO placeholder until I figure out what this trait needs to implement
#[allow(dead_code)] // TODO remove later when used
pub trait Hardware {
    fn apply_config(&mut self, config: &GPIOConfig);
    fn get_state(&self) -> GPIOState;
}