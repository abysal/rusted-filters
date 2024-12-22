use serde::Deserialize;
use std::path::Path;
use strum::*;

#[derive(Deserialize, Debug, Clone, Copy, Display, PartialEq)]
pub enum ParserMode {
    #[strum(serialize = "ts")]
    TS,
    #[strum(serialize = "js")]
    JS,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExposedOfficeConfig {
    pub mode: ParserMode,
    pub type_check: bool,
}

impl Default for ExposedOfficeConfig {
    fn default() -> Self {
        Self {
            mode: ParserMode::TS,
            type_check: false,
        }
    }
}

#[derive(Clone)]
pub struct OfficeConfig {
    pub parsed_config: ExposedOfficeConfig,
    pub script_entry: String,
    pub script_search_location: Box<Path>,
}
