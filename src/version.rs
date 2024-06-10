const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = "piggui";
const LICENSE: &str = env!("CARGO_PKG_LICENSE");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

#[must_use]
pub fn version() -> String {
    format!(
        "{name} {version}\n\
        Copyright (C) 2024 The {name} Developers \n\
        License {license}: <https://www.gnu.org/licenses/{license_lower}.html>\n\
        This is free software: you are free to change and redistribute it.\n\
        There is NO WARRANTY, to the extent permitted by law.\n\
        \n\
        Written by the {name} Contributors.\n\
        Full source available at: {repository}",
        name = NAME,
        version = VERSION,
        license = LICENSE,
        license_lower = LICENSE.to_lowercase(),
        repository = REPOSITORY,
    )
}
