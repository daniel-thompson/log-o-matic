use std::{error::Error, io, net::IpAddr};

use clap::Parser;
use rust_tuyapi::{Payload, TuyaDevice};
use serde_json::json;

#[derive(Debug, Parser)]
pub struct Command {
    #[arg(long)]
    dry_run: bool,

    my_flame: u8,
}

pub fn main(ip_addr: IpAddr, local_key: &str, args: Command) -> Result<(), Box<dyn Error>> {
    if args.my_flame < 1 || args.my_flame > 9 {
        return Err(io::Error::other("Preset is not in range (1 to 9)").into());
    }
    let cmd = json!({
        "dps": {
            "102": args.my_flame,
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
