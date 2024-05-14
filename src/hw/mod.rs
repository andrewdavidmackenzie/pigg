use crate::gpio::{GPIOConfig, GPIOState};

#[cfg_attr(feature = "pi", path="pi.rs")]
#[cfg_attr(feature = "pico", path="pico.rs")]
#[cfg_attr(not(any(feature = "pico", feature = "pico")), path="none.rs")]
pub mod implementation;

pub fn get() -> impl Hardware {
    implementation::NoneHW::get()
}

// TODO placeholder until I figure out what this trait needs to implement
pub trait Hardware {
    fn apply_config(&mut self, config: &GPIOConfig);
    fn get_state(&self) -> GPIOState;
}