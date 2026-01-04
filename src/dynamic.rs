use std::time::Duration;
use std::{
    cmp::{max, min},
    error::Error,
    net::IpAddr,
    thread,
};

use chrono::{DateTime, Local, MappedLocalTime, NaiveTime, Timelike};
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

fn get_next_occurrence(current_dt: &DateTime<Local>, target_time: NaiveTime) -> DateTime<Local> {
    let candidate = current_dt
        .date_naive()
        .and_time(target_time)
        .and_local_timezone(Local);

    let target_dt = match candidate {
        MappedLocalTime::Single(dt) => dt,
        MappedLocalTime::Ambiguous(dt1, dt2) => dt1,
        MappedLocalTime::None => {
            panic!("Local time does not exist (gap)");
        }
    };

    if &target_dt <= current_dt {
        target_dt + chrono::Duration::days(1)
    } else {
        target_dt
    }
}

pub fn main(ip_addr: IpAddr, local_key: &str, args: Command) -> Result<(), Box<dyn Error>> {
    let mut now = chrono::Local::now();
    now = now
        - chrono::Duration::seconds(now.second() as i64)
        - chrono::Duration::nanoseconds(now.nanosecond() as i64);

    let start = now - chrono::Duration::minutes(args.age);
    let bed_time = get_next_occurrence(&start, chrono::NaiveTime::from_hms_opt(22, 45, 0).unwrap());

    loop {
        let age = now.signed_duration_since(start).num_minutes();
        let remaining = bed_time.signed_duration_since(now).num_minutes();
        let offset = now.minute();
        let time_since_log = std::cmp::min(age, offset as i64);

        if remaining <= -50 {
            break;
        }

        let bed_temp = if remaining >= 0 {
            min(age * 2, 100)
        } else {
            (remaining + 50) * 2
        } as u8;

        let fuel_level = max(
            1,
            if remaining <= 60 {
                // Let's bank up the fire an hour before we want it to die down
                (remaining * 3) / 2
            } else {
                100 - ((time_since_log * 3) / 2)
            },
        ) as u8;

        let draw = if age < 50 {
            50
        } else if remaining < 30 {
            1
        } else {
            33
        };

        println!(
            "{}: bed_temp {:3}  fuel_level {:3}  draw {:3}  age {:3}  remaining {:3}  time_since_log {:3}",
            now, bed_temp, fuel_level, draw, age, remaining, time_since_log
        );

        let flame = Flame::summon_fire(min(age * 2, 100) as u8, fuel_level, draw);

        let then = now + chrono::Duration::minutes(1);
        if !args.dry_run {
            let e = update_avanti(ip_addr, local_key, &flame);
            match e {
                Ok(_) => {
                    let sleep = then.signed_duration_since(chrono::Local::now());
                    thread::sleep(sleep.to_std().unwrap_or(std::time::Duration::from_secs(60)));
                }
                Err(e) => println!("{e}"),
            };
        }
        now = then;
    }

    Ok(())
}
