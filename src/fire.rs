use std::{error::Error, net::IpAddr};

use clap::Parser;
use log_o_matic::Flame;
use rust_tuyapi::{Payload, TuyaDevice};
use serde_json::json;

#[derive(Debug, Parser)]
pub struct Command {
    /// Show the JSON command but do not issue it to the fire
    #[arg(long)]
    dry_run: bool,

    /// Temperature of the fuel bed, in percent of maximum temp
    bed_temperature: u8,

    /// Fuel level, in percent
    fuel_level: u8,

    /// Air draw, in percent
    draw: u8,
}

pub fn main(ip_addr: IpAddr, local_key: &str, args: Command) -> Result<(), Box<dyn Error>> {
    let flame = Flame::summon_fire(args.bed_temperature, args.fuel_level, args.draw);
    let cmd = json!({
        "dps": {
            "104": flame.to_base64(),
        }
    });
    let payload = Payload::String(cmd.to_string());

    if args.dry_run {
        println!("{:?}", flame);
        println!("{}", payload);
        return Ok(());
    }

    let tuya_device = TuyaDevice::create("ver3.3", Some(local_key), ip_addr)?;
    tuya_device.set_new(payload, 0)?;

    Ok(())
}
