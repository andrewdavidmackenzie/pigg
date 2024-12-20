//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, io};

const SSID_FILE_NAME: &str = "ssid.toml";

#[derive(Serialize, Deserialize, Debug)]
struct SsidSpec {
    ssid_name: String,
    ssid_pass: String,
    security: String,
}

fn read_ssid(ssid_filename: &str) -> Result<SsidSpec, io::Error> {
    let ssid_string = std::fs::read_to_string(ssid_filename).map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could read {} file", ssid_filename),
        )
    })?;
    toml::from_str(&ssid_string)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not parse toml ssid file"))
}

/// Given an optional override of SSID details,generate that as a source file in OUT_DIR
fn generate_ssid(filename: &str, ssid: Option<SsidSpec>) -> io::Result<()> {
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out);
    let out_file = out_dir.join(filename);
    let mut file = File::create(out_file)?;

    file.write_all(
        b"\
#[allow(unused_imports)]\n
use heapless::String;\n
use crate::hw_definition::description::SsidSpec;\n\
#[allow(unused_imports)]\n
use core::str::FromStr;\n",
    )?;

    match ssid {
        None => file.write_all(
            b"\n\
pub(crate) fn get_default_ssid_spec() -> Option<SsidSpec> {{ \n\
    None \n\
}}",
        ),
        Some(spec) => file.write_all(
            format!(
                "\n\
pub(crate) fn get_default_ssid_spec() -> Option<SsidSpec> {{ \n\
    Some(SsidSpec {{ \n\
        ssid_name: String::from_str(\"{}\").unwrap(), \n\
        ssid_pass: String::from_str(\"{}\").unwrap(), \n\
        ssid_security: String::from_str(\"{}\").unwrap(), \n\
    }}) \n\
}}",
                spec.ssid_name, spec.ssid_pass, spec.security
            )
            .as_bytes(),
        ),
    }
}

fn main() -> io::Result<()> {
    // Put `memory.x` in our output directory and ensure it's on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed={}", SSID_FILE_NAME);

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    let ssid_spec = read_ssid(SSID_FILE_NAME).ok();

    generate_ssid("ssid.rs", ssid_spec)?;

    Ok(())
}
