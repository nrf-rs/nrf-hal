use std::{env, process};

fn main() {
    if env::var_os("CARGO_FEATURE_51").is_none() {
        // Not building for the nRF51. 52+ use too many interrupts for the thumbv6 target, so detect
        // that early.
        if env::var("TARGET").unwrap() == "thumbv6m-none-eabi" {
            eprintln!(
                "this nRF device does not support the `thumbv6m-none-eabi` target; \
                build for `thumbv7em-none-eabi(hf)` instead"
            );
            process::exit(1);
        }
    }
}
