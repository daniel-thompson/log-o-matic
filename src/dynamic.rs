use std::{cmp::min, error::Error, net::IpAddr, thread, time::Duration};

use chrono::Timelike;
use clap::Parser;
use log_o_matic::*;
use rust_tuyapi::{Payload, TuyaDevice};

#[derive(Debug, Parser)]
pub struct Command {
    /// Show how the fire would be managed
    #[arg(long)]
    dry_run: bool,

    /// Specify how long (in minutes) the fire has been lit for
    #[arg(long, default_value_t = 0)]
    age: i64,
}

fn update_avanti(ip_addr: IpAddr, local_key: &str, flame: &Flame) -> Result<(), Box<dyn Error>> {
    let tuya_device = TuyaDevice::create("ver3.3", Some(local_key), ip_addr)?;

    let payload = Payload::String(format!(
        "{{\"dps\":{{\"104\": \"{}\"}}}}",
        flame.to_base64()
    ));

    tuya_device.set_new(payload, 0)?;

    Ok(())
}

pub fn main(ip_addr: IpAddr, local_key: &str, args: Command) -> Result<(), Box<dyn Error>> {
    let start = chrono::Local::now();
    let bed_time = chrono::NaiveTime::from_hms_opt(22, 45, 0).unwrap();
    let mut now = chrono::Local::now();

    loop {
        if args.dry_run {
            now += chrono::Duration::minutes(2);
        } else {
            now = chrono::Local::now();
        }
        let target_today = now
            .date_naive()
            .and_time(bed_time)
            .and_local_timezone(chrono::Local)
            .unwrap();
        let age = now.signed_duration_since(start).num_minutes() + args.age;
        let remaining = target_today.signed_duration_since(now).num_minutes();
        let offset = now.minute();

        let mut time_since_log = std::cmp::min(age, offset as i64);
        // TODO: this doesn't work... we need the threshold to be the minutes
        //       value for "bed time"
        if remaining < 60 {
            time_since_log += 60;
        }

        if remaining <= -50 {
            break;
        }

        let fuel_level: u8 = if time_since_log > 99 {
            1
        } else {
            100 - time_since_log as u8
        };
        let draw = if age < 50 {
            50
        } else if remaining < 30 {
            1
        } else {
            33
        };

        let bed_temp = if remaining >= 0 {
            min(age * 2, 100)
        } else {
            ((remaining + 50) * 2)
        } as u8;

        println!(
            "{}: bed_temp {}  fuel_level {}  draw {}  age {}  remaining {}  time_since_log {}",
            now, bed_temp, fuel_level, draw, age, remaining, time_since_log
        );

        let flame = Flame::summon_fire(min(age * 2, 100) as u8, fuel_level, draw);

        if !args.dry_run {
            let e = update_avanti(ip_addr, local_key, &flame);
            match e {
                Ok(_) => thread::sleep(Duration::from_secs(120)),
                Err(e) => println!("{e}"),
            };
        }
    }

    Ok(())
}
