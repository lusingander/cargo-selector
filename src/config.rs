use std::{env, path::PathBuf};

use ratatui::style::Color;
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

pub struct ColorTheme {
    pub bg: Color,

    pub action_run_bg: Color,
    pub action_run_fg: Color,
    pub action_build_bg: Color,
    pub action_build_fg: Color,

    pub input_fg: Color,
    pub numbers_fg: Color,

    pub kind_fg: Color,
    pub name_fg: Color,
    pub name_match_fg: Color,
    pub path_fg: Color,
    pub features_fg: Color,

    pub selected_bg: Color,
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            bg: Color::Reset,

            action_run_bg: Color::Green,
            action_run_fg: Color::Black,
            action_build_bg: Color::Blue,
            action_build_fg: Color::Black,

            input_fg: Color::Reset,
            numbers_fg: Color::DarkGray,

            kind_fg: Color::Blue,
            name_fg: Color::White,
            name_match_fg: Color::Red,
            path_fg: Color::DarkGray,
            features_fg: Color::DarkGray,

            selected_bg: Color::Yellow,
        }
    }
}
