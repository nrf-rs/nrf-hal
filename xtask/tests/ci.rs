use std::env;
use std::{fs, process::Command};
use xtask::{EXAMPLES, HALS};

fn build_example(example: &str, target: &str, feature: Option<&str>) {
    println!("building `{}` for `{}`", example, target);
    let mut cargo = Command::new("cargo");
    let toml_path = format!("examples/{}/Cargo.toml", example);
    cargo.args(&["build", "--target", target, "--manifest-path", &toml_path]);
    if let Some(feature) = feature {
        cargo.args(&["--features", feature]);
    }

    let status = cargo
        .status()
        .map_err(|e| format!("could not execute {:?}: {}", cargo, e))
        .unwrap();
    assert!(
        status.success(),
        "command exited with error status: {:?}",
        cargo
    );
}

fn main() {
    xtask::install_targets();

    // We execute from the `xtask` dir, so `cd ..` so that we can find `examples` etc.
    env::set_current_dir("..").unwrap();

    // Make sure all the tomls are formatted in a way that's compatible with our tooling.
    xtask::bump_versions("0.0.0", true);

    // Build-test every HAL.
    for (hal, target) in HALS {
        let mut cargo = Command::new("cargo");
        let toml_path = format!("{}/Cargo.toml", hal);
        let status = cargo
            .args(&["build", "--manifest-path", &toml_path, "--target", target])
            .status()
            .map_err(|e| format!("could not execute {:?}: {}", cargo, e))
            .unwrap();
        assert!(
            status.success(),
            "command exited with error status: {:?}",
            cargo
        );
    }

    // Build-test every example with each supported feature.
    for (example, features) in EXAMPLES {
        // Features are exclusive (they select the target chip), so we test each one
        // individually.
        if features.is_empty() {
            // Use a default target.
            build_example(example, "thumbv7em-none-eabihf", None);
        } else {
            for feature in *features {
                let target = xtask::feature_to_target(feature);
                build_example(example, target, Some(*feature));
            }
        }
    }

    // Ensure that no examples get added without an entry in EXAMPLES.
    for entry in fs::read_dir("examples").unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_str().unwrap();

        if EXAMPLES
            .iter()
            .find(|(example, ..)| *example == name)
            .is_none()
        {
            panic!("example `{}` is missing an entry in xtask `EXAMPLES`", name);
        }
    }
}
