use std::{
    fs::File,
    io::{self, BufReader},
    net::ToSocketAddrs,
    path::Path,
    process,
};

use clap::{Parser, Subcommand};
use log::error;
use serde::Deserialize;

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

    /// Only report warnings and errors
    #[arg(short, long)]
    quiet: bool,

    /// Increase verbosity
    #[arg(short, long)]
    verbose: bool,
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

#[derive(Debug, Deserialize)]
struct TuyaRaw {
    result: Vec<TuyaDevice>,
}

#[derive(Debug, Deserialize)]
struct TuyaDevice {
    name: String,
    local_key: String,
}

fn app() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let default_level = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "warn"
    } else {
        "warn,log_o_matic=info"
    };
    let env = env_logger::Env::default().default_filter_or(default_level);
    env_logger::Builder::from_env(env).init();

    let home = std::env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Bad environment"))?;
    let filename = "tuya-raw.json";
    let file = if let Ok(f) = File::open(Path::new(&home).join(".log-o-matic").join(filename)) {
        f
    } else {
        File::open(filename)?
    };
    let reader = BufReader::new(file);
    let tuya_raw: TuyaRaw = serde_json::from_reader(reader)?;
    let avanti = tuya_raw
        .result
        .iter()
        .find(|device| device.name == "AVANTI")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Device 'AVANTI' not found"))?;

    let local_key = &avanti.local_key;
    let ip_addr = ("TY_WR", 0)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "TY_WR not found"))?
        .ip();

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
