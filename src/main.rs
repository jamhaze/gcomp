use gcomp::Config;
use std::process;
use clap::Parser;

fn main() {
    let config = Config::parse();

    if let Err(e) = gcomp::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    };
}
