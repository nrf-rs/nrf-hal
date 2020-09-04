use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let subcommand = args.next();
    match subcommand.as_deref() {
        Some("bump") => {
            let new_version = args.next().expect("missing <semver> argument");
            xtask::bump_versions(&new_version, false);
        }
        _ => {
            eprintln!("usage: cargo xtask <subcommand>");
            eprintln!();
            eprintln!("subcommands:");
            eprintln!("    bump <semver> - bump crate versions to <semver>");
        }
    }
}
