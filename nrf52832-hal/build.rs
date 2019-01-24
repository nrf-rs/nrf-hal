use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let linker = match (cfg!(feature = "xxAA-package"), cfg!(feature = "xxAB-package")) {
        (false, false) | (true, true) => {
            panic!("\n\nMust select exactly one package for linker script generation!\nChoices: 'xxAA-package' or 'xxAB-package'\n\n");
        }
        (true, false) => {
            include_bytes!("memory_xxAA.x")
        }
        (false, true) => {
            include_bytes!("memory_xxAB.x")
        }
    };

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(linker)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=memory_xxAA.x");
    println!("cargo:rerun-if-changed=memory_xxAB.x");
}
