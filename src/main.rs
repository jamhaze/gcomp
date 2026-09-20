use genetic_compression::Cli;
use std::process;
use clap::Parser;

fn main() {
    let config = Cli::parse();

    if let Err(e) = genetic_compression::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    };
}
