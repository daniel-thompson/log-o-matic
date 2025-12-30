use std::{net::IpAddr, process, str::FromStr};

use clap::{Parser, Subcommand};
use log::error;

mod dynamic;
mod fire;
mod power;
mod preset;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Dynamically manage the fire for a cosy evening
    Dynamic(dynamic::Command),

    /// Set the flame level using real-world units
    Fire(fire::Command),

    /// Turn the fire on and off
    Power(power::Command),

    /// Choose a specific Flame from the available presets
    Preset(preset::Command),
}

fn app() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let env = env_logger::Env::default().default_filter_or("warn");
    env_logger::Builder::from_env(env).init();

    let ip_addr = IpAddr::from_str("192.168.1.158")?;
    let local_key = "#x?35]L*|u_m;Bv_";

    match cli.command {
        Commands::Dynamic(args) => dynamic::main(ip_addr, local_key, args),
        Commands::Fire(args) => fire::main(ip_addr, local_key, args),
        Commands::Power(args) => power::main(ip_addr, local_key, args),
        Commands::Preset(args) => preset::main(ip_addr, local_key, args),
    }
}

fn main() {
    if let Err(e) = app() {
        error!("{e}");
        process::exit(1);
    }
}
