use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    if let Some((flash, mem)) = memory_sizes() {
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

        let mut file = File::create(out.join("memory.x")).unwrap();

        write!(
            file,
            r#"MEMORY
{{
FLASH : ORIGIN = 0x00000000, LENGTH = {}
RAM : ORIGIN = 0x20000000, LENGTH = {}
}}
"#,
            flash, mem
        )
        .unwrap();

        println!("cargo:rustc-link-search={}", out.display());
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn memory_sizes() -> Option<(&'static str, &'static str)> {
    match (
        cfg!(feature = "xxAA-package"),
        cfg!(feature = "xxAB-package"),
        cfg!(feature = "xxAC-package"),
    ) {
        // Allow users to provide their own memory.x by disabling all features
        (false, false, false) => None,
        (true, false, false) => Some(("256K", "16K")),
        (false, true, false) => Some(("128K", "16K")),
        (false, false, true) => Some(("256K", "32K")),
        _ => panic!("Multiple memory configuration features specified"),
    }
}
