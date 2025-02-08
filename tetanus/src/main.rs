#![allow(dead_code)]
use rusted_office::config::{ExposedOfficeConfig, OfficeConfig};
use rusted_office::filter::RustedOffice;
use rusted_rotation::Rotation;
use rusted_shut::addon::addon::Addon;
use rusted_shut::addon::custom_infrastructure::addon_processor::AddonProcessor;
use rusted_shut::addon::custom_infrastructure::component::custom_block::EmptyBlockState;
use rusted_shut::parser::addon_parser::{AddonParser, ParserConfig};
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn default_rp() -> String {
    "RP".to_string()
}

fn default_bp() -> String {
    "BP".to_string()
}

fn default_data() -> String {
    "data".to_string()
}

fn default_script_path() -> String {
    "data/gametests/src".to_string()
}

fn default_main_file() -> String {
    "main".to_string()
}

fn default_base_path() -> String {
    "./".into()
}

fn true_func() -> bool {
    true
}
fn false_func() -> bool {
    false
}

#[derive(Debug, Deserialize)]
struct TetanusConfig {
    #[serde(default = "true_func")]
    enable_rotation_filter: bool,
    #[serde(default = "true_func")]
    enable_office: bool,
    #[serde(default = "default_rp")]
    rp_path: String,
    #[serde(default = "default_bp")]
    bp_path: String,
    #[serde(default = "default_data")]
    data_path: String,
    #[serde(default = "default_script_path")]
    script_path: String,
    #[serde(default = "default_base_path")]
    base_path: String,
    #[serde(default = "default_main_file")]
    main_file: String,
}

impl TetanusConfig {
    pub fn real_script_dir(&self) -> Box<Path> {
        let mut path = PathBuf::from(self.base_path.clone());
        path.push(&self.script_path);
        path.into_boxed_path()
    }

    pub fn real_main(&self) -> String {
        format!("{}.{}", self.main_file, "ts")
    }
}

fn get_config() -> Result<TetanusConfig, serde_json::Error> {
    let s = std::env::args().collect::<Vec<_>>();
    let idx = s.len() - 1;
    if let Some(v) = s.get(idx) {
        Ok(serde_json::from_str(v).unwrap_or(serde_json::from_str("{}").unwrap()))
    } else {
        serde_json::from_str("{}")
    }
}

fn apply_rotation(addon: Addon) -> Addon {
    let mut processor =
        AddonProcessor::<serde_json::Error, serde_json::Error, EmptyBlockState>::new(
            EmptyBlockState,
        );

    processor.bind_block_component(Rotation);
    
    processor
        .process_addon(addon)
        .expect("Failed to apply rotations!")
}

fn apply_office(addon: Addon, config: &TetanusConfig) -> Addon {
    let config = OfficeConfig {
        parsed_config: ExposedOfficeConfig::default(),
        script_entry: config.real_main(),
        script_search_location: config.real_script_dir(),
    };

    let filter = RustedOffice::new(config);
    let addon = filter.process(addon).expect("Failed to apply office!");

    addon
}

fn main() {
    let conf = get_config().expect("Failed to process config");

    let mut addon = AddonParser::parse_addon(
        conf.base_path.clone(),
        ParserConfig::builder()
            .skip_bland(true)
            .parse_items(true)
            .parse_block(true)
            .bp_from_base(conf.bp_path.clone())
            .rp_from_base(conf.rp_path.clone())
            .build(),
    )
    .expect(&format!("Failed to parse addon: Config: {conf:?}"));

    if conf.enable_rotation_filter {
        addon = apply_rotation(addon);
    }

    if conf.enable_office {
        let old_wd = std::env::current_dir().unwrap();
        std::env::set_current_dir(
            Path::new(&conf.base_path)
                .canonicalize()
                .expect("Base path does not exist!"),
        )
        .expect("Failed to change into the addon directory");
        addon = apply_office(addon, &conf);

        std::env::set_current_dir(&old_wd).expect("Failed to change into the old directory");
    }

    addon.write().unwrap()
}
