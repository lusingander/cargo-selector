use std::{env, path::PathBuf};

use serde::Deserialize;

use crate::MatchType;

const CONFIG_PATH_ENV_VAR: &str = "CARGO_SELECTOR_CONFIG";

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub match_type: Option<MatchType>,
}

impl Config {
    pub fn load() -> Config {
        if let Some(path) = config_file_path() {
            let content = std::fs::read_to_string(path).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Config::default()
        }
    }
}

fn config_file_path() -> Option<PathBuf> {
    env::var(CONFIG_PATH_ENV_VAR).map(PathBuf::from).ok()
}
