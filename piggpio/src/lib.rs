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
mod tests;

/// Create a new HW instance - should only be called once
pub fn get_hardware() -> Option<HW> {
    // debug build - Pi or Non-Pi Hardware
    #[cfg(debug_assertions)]
    return Some(HW::new(env!("CARGO_PKG_NAME")));

    // release build - Not Pi hardware
    #[cfg(all(
        not(debug_assertions),
        not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))
    ))]
    return None;

    // release build - Pi hardware
    #[cfg(all(
        not(debug_assertions),
        all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        )
    ))]
    return Some(HW::new(env!("CARGO_PKG_NAME")));
}
