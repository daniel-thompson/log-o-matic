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
