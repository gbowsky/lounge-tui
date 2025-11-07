use chrono::Utc;
use confy::ConfyError;
use serde_derive::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct LoungeConfig {
    pub group_id: String,
    pub level_id: String,
    pub pin: String,
    pub last_name: String,
    pub setup_passed: bool,
    pub selected_date: i64,
    pub theme: u8,
}

impl ::std::default::Default for LoungeConfig {
    fn default() -> Self {
        Self {
            group_id: "".to_string(),
            level_id: "".to_string(),
            pin: "".to_string(),
            last_name: "".to_string(),
            setup_passed: false,
            selected_date: Utc::now().timestamp(),
            theme: 0,
        }
    }
}

pub fn get_config() -> Result<LoungeConfig, ConfyError> {
    match env::var("SSH_CONNECTION") {
        Ok(val) => {
            let val: &str = val.split_whitespace().next().unwrap();
            confy::load("lounge-tui", val)
        }
        Err(_e) => confy::load("lounge-tui", None),
    }
}

pub fn store_config(cfg: LoungeConfig) -> Result<(), ConfyError> {
    match env::var("SSH_CONNECTION") {
        Ok(val) => {
            let val: &str = val.split_whitespace().next().unwrap();
            confy::store("lounge-tui", val, cfg)
        }
        Err(_e) => confy::store("lounge-tui", None, cfg),
    }
}

pub fn get_store_path() -> Result<PathBuf, ConfyError> {
    match env::var("SSH_CONNECTION") {
        Ok(val) => {
            let val: &str = val.split_whitespace().next().unwrap();
            confy::get_configuration_file_path("lounge-tui", val)
        }
        Err(_e) => confy::get_configuration_file_path("lounge-tui", None),
    }
}
