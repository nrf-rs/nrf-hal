use std::process::Command;

pub static HALS: &[(&str, &str)] = &[
    ("nrf51-hal", "thumbv6m-none-eabi"),
    ("nrf9160-hal", "thumbv8m.main-none-eabihf"),
    ("nrf52810-hal", "thumbv7em-none-eabi"),
    ("nrf52832-hal", "thumbv7em-none-eabihf"),
    ("nrf52833-hal", "thumbv7em-none-eabihf"),
    ("nrf52840-hal", "thumbv7em-none-eabihf"),
];

pub static EXAMPLES: &[(&str, &[&str])] = &[
    ("ccm-demo", &["52810", "52832", "52833", "52840"]),
    ("comp-demo", &[]),
    ("ecb-demo", &["51", "52810", "52832", "52833", "52840"]),
    ("gpiote-demo", &[]),
    ("i2s-controller-demo", &[]),
    ("i2s-peripheral-demo", &[]),
    ("lpcomp-demo", &[]),
    ("ppi-demo", &["51", "52810", "52832", "52833", "52840"]),
    ("pwm-demo", &[]),
    ("qdec-demo", &[]),
    ("rtic-demo", &["51", "52810", "52832", "52840"]),
    ("spi-demo", &[]),
    ("twi-ssd1306", &["52832", "52840"]),
    ("twim-demo", &[]),
    ("twis-demo", &[]),
    ("wdt-demo", &[]),
];

pub fn feature_to_target(feat: &str) -> &str {
    match feat {
        "51" => "thumbv6m-none-eabi",
        "52810" => "thumbv7em-none-eabi",
        _ if feat.starts_with("52") => "thumbv7em-none-eabihf",
        _ => panic!("unknown Cargo feature `{}`", feat),
    }
}

pub fn install_targets() {
    let mut targets = HALS
        .iter()
        .map(|(_, target)| *target)
        .chain(
            EXAMPLES
                .iter()
                .flat_map(|(_, features)| features.iter().map(|feat| feature_to_target(feat))),
        )
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();

    let mut cmd = Command::new("rustup");
    cmd.args(&["target", "add"]).args(&targets);
    let status = cmd
        .status()
        .map_err(|e| format!("couldn't execute {:?}: {}", cmd, e))
        .unwrap();
    assert!(
        status.success(),
        "failed to install targets with rustup: {:?}",
        cmd
    );
}
