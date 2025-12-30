use std::{error::Error, net::IpAddr};

use clap::{Parser, ValueEnum};
use rust_tuyapi::{Payload, TuyaDevice};
use serde_json::json;

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Item {
    System,
    Flame,
    Heater,
}

#[derive(Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Action {
    On,
    Off,
}
#[derive(Debug, Parser)]
pub struct Command {
    /// Show the JSON command but do not issue it to the fire
    #[arg(long)]
    dry_run: bool,

    item: Item,
    action: Action,
}

pub fn main(ip_addr: IpAddr, local_key: &str, args: Command) -> Result<(), Box<dyn Error>> {
    let key = match args.item {
        Item::System => "101",
        Item::Flame => "10",
        Item::Heater => "1",
    };
    let value = args.action == Action::On;

    let cmd = json!({
        "dps": {
            key: value,
        }
    });
    let payload = Payload::String(cmd.to_string());

    if args.dry_run {
        println!("{}", payload);
        return Ok(());
    }

    let tuya_device = TuyaDevice::create("ver3.3", Some(local_key), ip_addr)?;
    tuya_device.set_new(payload, 0)?;

    Ok(())
}
