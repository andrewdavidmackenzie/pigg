use crate::driver::HW;

pub mod driver;
mod pin_descriptions;

/// Create a new HW instance - should only be called once
pub fn get() -> HW {
    HW::default()
}
