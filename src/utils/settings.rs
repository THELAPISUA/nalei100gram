use std::{
    fs::{self, File},
    io::{Read, Write},
};

use log::error;

use crate::utils::types::Settings;

pub fn load_settings() -> Option<Settings> {
    let f = File::open("data/settings.toml");
    match f {
        Ok(mut file) => {
            let mut config_text = String::new();
            let res = file.read_to_string(&mut config_text);
            match res {
                Ok(_) => {
                    let config: Result<Settings, toml::de::Error> =
                        toml::from_str(&config_text.as_str());
                    match config {
                        Ok(s) => Some(s),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}
pub fn save_settings(s: Settings) {
    if let Err(e) = fs::create_dir_all("data/") {
        error!("Failed to create 'data' directory: {}", e);
        panic!("{}", e);
    }
    let mut f = File::create("data/settings.toml").unwrap();
    let toml_string = toml::to_string_pretty(&s).unwrap();
    let _ = f.write_all(toml_string.as_bytes());
}
