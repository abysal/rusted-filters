use oxc::span::SourceType;
use serde::Deserialize;
use std::path::Path;
use strum::*;

#[derive(Deserialize, Debug, Clone, Copy, Display)]
pub enum ParserMode {
    #[strum(serialize = ".ts")]
    TS,
    #[strum(serialize = ".js")]
    JS,
}

impl From<ParserMode> for SourceType {
    fn from(value: ParserMode) -> Self {
        match value {
            ParserMode::TS => SourceType::ts(),
            ParserMode::JS => SourceType::mjs(),
        }
    }
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
    pub script_search_location: Box<Path>,
}
