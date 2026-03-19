use clap::{CommandFactory, Parser};

mod banner;
mod cli;
mod commands;
mod settings;
mod utils;

use banner::banner;
use cli::{Cli, Command};
use commands::{
    compress::run_compress,
    config::run_config,
    diff::run_diff,
    gains::run_gains,
    install::{run_install, run_uninstall},
    proxy::run_proxy,
    stats::run_stats,
    upgrade::run_upgrade,
};
use settings::load_config;

fn main() {
    let cli = Cli::parse();
    let cfg = load_config();

    match cli.command {
        Some(Command::Install(args))   => run_install(&args, &cfg),
        Some(Command::Uninstall(args)) => run_uninstall(&args, &cfg),
        Some(Command::Proxy(args))     => run_proxy(&args, &cfg),
        Some(Command::Compress(args))  => run_compress(&args, &cfg),
        Some(Command::Stats(args))     => run_stats(&args, &cfg),
        Some(Command::Diff(args))      => run_diff(&args, &cfg),
        Some(Command::Config)          => run_config(&cfg),
        Some(Command::Gains)           => run_gains(),
        Some(Command::Upgrade)         => run_upgrade(),
        None => {
            print!("{}", banner());
            println!();
            let _ = Cli::command().print_help();
            println!();
        }
    }
}
