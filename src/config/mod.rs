use anyhow::Result;
use serde::{Deserialize, Serialize};

use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::messages::Language;
use crate::update_language;

const APP_NAME: &str = "helferlein";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Config {
    pub(crate) data_folder: Option<PathBuf>,
    pub(crate) file_open_command: Option<String>,
    pub(crate) language: String,
}

pub(crate) fn load_config() -> Result<Config> {
    let config_file = check_config_exists()?;
    let mut file = File::open(&config_file)?;
    let mut buf = String::default();
    File::read_to_string(&mut file, &mut buf)?;
    let res: Config = toml::from_str(&buf)?;
    update_language(&res.language);
    Ok(res)
}

pub(crate) fn save_config(config: &Config) -> Result<()> {
    let config_file = check_config_exists()?;
    let serialized = toml::to_string(&config)?;
    let mut file = File::create(&config_file)?;
    file.write_all(serialized.as_bytes())?;
    update_language(&config.language);
    Ok(())
}

fn check_config_exists() -> Result<PathBuf> {
    let mut dir: PathBuf = dirs::config_dir().unwrap_or_else(|| "./".into());
    dir.push(APP_NAME);

    if !dir.exists() {
        create_dir_all(&dir)?;
    }
    dir.push(CONFIG_FILE);
    if !dir.exists() {
        let mut fd = File::create(&dir)?;
        let default_config = Config {
            data_folder: None,
            file_open_command: None,
            language: Language::EN.name().into(),
        };
        let serialized = toml::to_string(&default_config)?;
        fd.write_all(serialized.as_bytes())?;
    }
    Ok(dir)
}
