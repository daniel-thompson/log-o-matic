use std::{error::Error, net::IpAddr, thread, time::Duration};

use chrono::Timelike;
use clap::Parser;
use log_o_matic::*;
use rust_tuyapi::{Payload, TuyaDevice};

#[derive(Debug, Parser)]
pub struct Command {}

fn fire(age: u16, fuel_level: u8, draw: u8) -> Flame {
    let mut flame = Flame::new();

    // main flame brightness is controlled by draw and fuel level
    flame.main_flame.brightness = ((2 * fuel_level as u32 + 3 * draw as u32) / 5) as u8;
    // flame speed is entirely controlled by the draw
    flame.flame_speed = draw / 2;
    // TODO: quantize for least noise

    // flame palette brightness is controlled by draw and fuel level
    flame.flame_palette.brightness = ((3 * fuel_level as u32 + 2 * draw as u32) / 5) as u8;

    // fuel bed is controlled by age and draw
    let age_level = if age > 50 { 100 } else { age as u32 * 2 };
    flame.fuel_bed.brightness = ((age_level / 5) + (age_level * draw as u32) / 180) as u8;

    // glowing logs are controlled by age and draw (but more age)
    flame.glowing_logs.brightness = ((age_level / 2) + (age_level * draw as u32) / 200) as u8;

    // down light is controlled by the brightness of the rest of the fire
    flame.down_light.brightness = ((flame.main_flame.brightness as u32
        + flame.main_flame.brightness as u32
        + 2 * flame.fuel_bed.brightness as u32)
        / 4) as u8;

    //dbg!(&flame);
    flame
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

pub fn main(ip_addr: IpAddr, local_key: &str, _args: Command) -> Result<(), Box<dyn Error>> {
    let start = chrono::Local::now();
    let bed_time = chrono::NaiveTime::from_hms_opt(22, 45, 0).unwrap();

    loop {
        let now = chrono::Local::now();
        let target_today = now
            .date_naive()
            .and_time(bed_time)
            .and_local_timezone(chrono::Local)
            .unwrap();
        let age = now.signed_duration_since(start).num_minutes();
        let remaining = target_today.signed_duration_since(now).num_minutes();
        let offset = now.minute();

        let mut time_since_log = std::cmp::min(age, offset as i64);
        // TODO: this doesn't work... we need the threshold to be the minutes
        //       value for "bed time"
        if remaining < 60 {
            time_since_log += 60;
        }

        let fuel_level: u8 = if time_since_log > 99 {
            1
        } else {
            100 - time_since_log as u8
        };
        let draw = if age < 50 {
            50
        } else if remaining < 30 {
            10
        } else {
            33
        };

        println!(
            "{}: age {}  fuel_level {}  draw {}  remaining {}  time_since_log {}",
            now, age, fuel_level, draw, remaining, time_since_log
        );

        let flame = fire(age as u16, fuel_level, draw);

        let e = update_avanti(ip_addr, local_key, &flame);
        match e {
            Ok(_) => thread::sleep(Duration::from_secs(120)),
            Err(e) => println!("{e}"),
        };

        if remaining <= 0 {
            break;
        }
    }

    Ok(())
}
