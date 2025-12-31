use std::cmp::{max, min};

use base64::Engine;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    Red = 1,
    Orange = 2,
    Yellow = 3,
    Green = 4,
    Blue = 5,
    Purple = 6,
    Lilac = 7,
    White = 8,
    Rainbow = 9,
    OrangeRedGlow = 10,
    OrangeLilacGlow = 11,
    BlueLilacGlow = 12,
}

#[derive(Debug)]
pub struct PrimaryLedString {
    pub on: bool,
    pub color: Color,
    pub brightness: u8,
}

impl PrimaryLedString {
    fn to_u16(&self) -> u16 {
        ((self.on as u16) << 9) + ((self.color as u16) << 8) + self.brightness as u16
    }
}

#[derive(Debug)]
pub struct SecondaryLedString {
    pub on: bool,
    pub sync: bool,
    pub color: Color,
    pub brightness: u8,
}

impl SecondaryLedString {
    fn to_u16(&self) -> u16 {
        ((self.on as u16) << 10)
            + ((self.sync as u16) << 9)
            + ((self.color as u16) << 8)
            + self.brightness as u16
    }
}

#[derive(Debug)]
pub struct MoodLightLedString {
    pub on: bool,
    pub color: u8,
    pub brightness: u8,
}

impl MoodLightLedString {
    pub fn off() -> Self {
        Self {
            on: false,
            color: 1,
            brightness: 0,
        }
    }

    fn to_u16(&self) -> u16 {
        ((self.on as u16) << 9) + ((self.color as u16) << 8) + self.brightness as u16
    }
}

#[derive(Debug)]
pub struct Flame {
    pub main_flame: PrimaryLedString,
    pub flame_palette: SecondaryLedString,
    pub fuel_bed: PrimaryLedString,
    pub glowing_logs: SecondaryLedString,
    pub mood_light: MoodLightLedString,
    pub down_light: SecondaryLedString,
    pub flame_speed: u8,
}

impl Flame {
    pub fn new() -> Self {
        Self {
            main_flame: PrimaryLedString {
                on: true,
                color: Color::Yellow,
                brightness: 100,
            },
            flame_palette: SecondaryLedString {
                on: true,
                sync: false,
                color: Color::Red,
                brightness: 100,
            },
            fuel_bed: PrimaryLedString {
                on: true,
                color: Color::Red,
                brightness: 100,
            },
            glowing_logs: SecondaryLedString {
                on: true,
                sync: false,
                color: Color::OrangeRedGlow,
                brightness: 100,
            },
            mood_light: MoodLightLedString::off(),
            down_light: SecondaryLedString {
                on: true,
                sync: false,
                color: Color::Red,
                brightness: 100,
            },
            flame_speed: 60,
        }
    }

    /// Use "real world" parameters to dynamically configure the fire.
    ///
    /// The three parameters, each expressed as a percent, represent the
    /// temperature of the fuel bed, the fuel level and whether the fire is
    /// being allowed to draw in air (to control the burn rate).
    pub fn summon_fire(bed_temp: u8, fuel_level: u8, draw: u8) -> Self {
        let mut flame = Flame::new();

        let bed_temp = min(bed_temp as u32, 100);
        let fuel_level = min(fuel_level as u32, 100);
        let draw = min(draw as u32, 100);

        // main flame brightness is controlled by draw and fuel level
        flame.main_flame.brightness = ((2 * fuel_level + 3 * draw) / 5) as u8;
        // flame speed is entirely controlled by the draw
        flame.flame_speed = max(draw as u8 / 2, 1);
        // TODO: quantize for least noise

        // flame palette brightness is controlled by draw and fuel level
        flame.flame_palette.brightness = ((3 * fuel_level + 2 * draw) / 5) as u8;

        // fuel bed is controlled by bed temp and draw
        flame.fuel_bed.brightness = ((bed_temp / 5) + (bed_temp * draw) / 180) as u8;

        // glowing logs are controlled by bed temp and draw (but more bed temp)
        flame.glowing_logs.brightness = ((bed_temp / 2) + (bed_temp * draw) / 200) as u8;

        // down light is controlled by the brightness of the rest of the fire
        flame.down_light.brightness = ((flame.main_flame.brightness as u32
            + flame.main_flame.brightness as u32
            + 2 * flame.fuel_bed.brightness as u32)
            / 4) as u8;

        // The fuel bed is generally too bright (but right now I don't want to
        // change the downlight settings to let's just post-process to dim the
        // bed a little).
        flame.fuel_bed.brightness = (((flame.fuel_bed.brightness as u32) * 2) / 3) as u8;

        //dbg!(&flame);
        flame
    }

    pub fn to_base64(&self) -> String {
        let mut raw = [0u8; 15];
        raw[0..2].copy_from_slice(&self.main_flame.to_u16().to_be_bytes());
        raw[2..4].copy_from_slice(&self.flame_palette.to_u16().to_be_bytes());
        raw[4..6].copy_from_slice(&self.fuel_bed.to_u16().to_be_bytes());
        raw[6..8].copy_from_slice(&self.glowing_logs.to_u16().to_be_bytes());
        raw[8..10].copy_from_slice(&self.mood_light.to_u16().to_be_bytes());
        raw[10..12].copy_from_slice(&self.down_light.to_u16().to_be_bytes());
        raw[12..14].copy_from_slice(&0x0264_u16.to_be_bytes());
        raw[14] = self.flame_speed;

        base64::engine::general_purpose::STANDARD.encode(&raw)
    }
}
