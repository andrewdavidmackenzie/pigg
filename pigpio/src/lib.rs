/// There are two implementations of the `HW` struct.
///
#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
pub mod pi;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
pub use pi::HW;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
)))]
pub mod fake_pi;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
)))]
pub use fake_pi::HW;

mod pin_descriptions;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
)))]
/// Create a new HW instance - should only be called once
pub fn get_hardware() -> Option<HW> {
    Some(HW::default())
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
/// Create a new HW instance - should only be called once
pub fn get_hardware() -> Option<HW> {
    Some(HW::default())
}
