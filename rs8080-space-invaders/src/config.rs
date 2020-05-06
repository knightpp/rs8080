use sdl2::keyboard::Keycode;
use serde::Deserialize;
use std::fmt::{self, Formatter};
use std::fs::File;
use std::{io::Read, ops::Deref};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) controls: Controls,
    pub volume: Sound,
}

pub(crate) fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Debug, Copy, Clone)]
pub struct Keycodee(Keycode);

impl<'de> Deserialize<'de> for Keycodee {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        let kc = Keycode::from_name(&s);
        if let Some(x) = kc {
            Ok(Keycodee(x))
        } else {
            Err(D::Error::custom(format!(
                "err in '{}', cannot parse as SDL2 Keycode",
                &s
            )))
        }
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}

impl Deref for Keycodee {
    type Target = Keycode;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Deserialize, Clone)]
pub(crate) struct Controls {
    pub(crate) insert_coin: Keycodee,
    pub(crate) start_1p: Keycodee,
    pub(crate) start_2p: Keycodee,
    pub(crate) shot_1p: Keycodee,
    pub(crate) shot_2p: Keycodee,
    pub(crate) left_1p: Keycodee,
    pub(crate) left_2p: Keycodee,
    pub(crate) right_1p: Keycodee,
    pub(crate) right_2p: Keycodee,
}

#[derive(Deserialize, Copy, Clone)]
pub(crate) struct Sound {
    pub volume: u8,
}

#[derive(Copy, Clone)]
pub(crate) struct ParseKeycodeErr {}
impl std::fmt::Display for ParseKeycodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse keycode from string")
    }
}
impl std::fmt::Debug for ParseKeycodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse keycode from string")
    }
}
impl std::error::Error for ParseKeycodeErr {}
