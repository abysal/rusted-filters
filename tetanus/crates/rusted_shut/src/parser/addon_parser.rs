use crate::addon::addon::Addon;
use crate::addon::blocks::block::Block;
use crate::addon::component::{ComponentError, FormattedComponentRegister};
use crate::addon::items::item::Item;
use crate::addon::path_resolver::AddonPathResolver;
use crate::addon::traits::FormattedJsonSerialize;
use crate::parser::addon_parser::AddonParseError::{FSError, JsonError};
use bon::Builder;
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct ParsedAddonResolver {
    block_path_lookup: HashMap<String, Box<Path>>,
    item_path_lookup: HashMap<String, Box<Path>>,
    base: PathBuf,
    rp_from_base: PathBuf,
    bp_from_base: PathBuf,
}

impl AddonPathResolver for ParsedAddonResolver {
    fn get_behaviour_block_output(&mut self, id: &str) -> PathBuf {
        let mut x = self.get_behaviour_block_base().clone();
        if let Some(r) = self.block_path_lookup.get(id) {
            x.push(r.to_path_buf());
            x
        } else {
            self.get_behaviour_block_base()
                .with_file_name(format!("{}.json", id.replace(":", "_")))
        }
    }

    fn get_behaviour_item_output(&mut self, id: &str) -> PathBuf {
        let mut x = self.get_behaviour_item_base().clone();
        if let Some(r) = self.item_path_lookup.get(id) {
            x.push(r.to_path_buf());
            x
        } else {
            self.get_behaviour_block_base()
                .with_file_name(format!("{}.json", id.replace(":", "_")))
        }
    }

    fn get_behaviour_base(&mut self) -> PathBuf {
        let mut r = self.base.clone();
        r.push(&self.bp_from_base);
        r
    }

    fn get_resource_base(&mut self) -> PathBuf {
        let mut r = self.base.clone();
        r.push(&self.rp_from_base);
        r
    }
}

impl ParsedAddonResolver {
    pub fn new(base: PathBuf, bp_from_base: PathBuf, rp_from_base: PathBuf) -> Self {
        Self {
            base,
            block_path_lookup: HashMap::new(),
            item_path_lookup: HashMap::new(),
            bp_from_base,
            rp_from_base,
        }
    }
}

#[derive(Builder)]
pub struct ParserConfig {
    parse_block: bool,
    parse_items: bool,
    skip_bland: bool,
    rp_from_base: Option<String>,
    bp_from_base: Option<String>,
    block_register: Option<FormattedComponentRegister>,
    item_register: Option<FormattedComponentRegister>,
}

#[derive(Error, Debug)]
pub enum AddonParseError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FSError(#[from] std::io::Error),
    #[error(transparent)]
    ComponentError(#[from] ComponentError),
}

pub struct AddonParser;

impl AddonParser {
    pub fn parse_addon<P: AsRef<Path>>(
        folder_base: P,
        config: ParserConfig,
    ) -> Result<Addon, AddonParseError> {
        let mut resolver = ParsedAddonResolver::new(
            folder_base.as_ref().to_path_buf(),
            config
                .bp_from_base
                .as_ref()
                .unwrap_or(&"BP".to_string())
                .into(),
            config
                .rp_from_base
                .as_ref()
                .unwrap_or(&"RP".to_string())
                .into(),
        );

        let blocks = Self::parse_blocks(&mut resolver, &config)?;
        let items = Self::parse_items(&mut resolver, &config)?;

        let mut addon = Addon::new(resolver);

        if let Some(v) = blocks {
            for b in v {
                addon.push_block(b)
            }
        }
        if let Some(v) = items {
            for i in v {
                addon.push_item(i)
            }
        }

        Ok(addon)
    }

    fn parse_blocks(
        resolver: &mut ParsedAddonResolver,
        parser_config: &ParserConfig,
    ) -> Result<Option<Vec<Block>>, AddonParseError> {
        let mut blocks = vec![];

        if !parser_config.parse_block {
            return Ok(None);
        }

        let base_path = resolver.get_behaviour_block_base();

        for file in WalkDir::new(resolver.get_behaviour_block_base())
            .into_iter()
            .filter_map(|e| {
                let e = e.ok();
                if let Some(e) = e {
                    if e.file_type().is_file() {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        {
            let data = std::fs::read_to_string(file.path()).map_err(|e| FSError(e))?;
            let raw_json = &serde_json::from_str::<Value>(&data).map_err(|e| JsonError(e))?;

            let blk = Block::from_json(
                raw_json,
                parser_config
                    .block_register
                    .as_ref()
                    .unwrap_or(&FormattedComponentRegister::init_blocks()),
                Version::new(0, 0, 0),
            )
            .map_err(|e| AddonParseError::ComponentError(e))?;

            if parser_config.skip_bland && blk.is_bland() {
                continue;
            }

            resolver.block_path_lookup.insert(
                blk.description.identifier.clone(),
                file.path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_path_buf()
                    .into_boxed_path(),
            );

            blocks.push(blk);
        }

        Ok(Some(blocks))
    }

    fn parse_items(
        resolver: &mut ParsedAddonResolver,
        parser_config: &ParserConfig,
    ) -> Result<Option<Vec<Item>>, AddonParseError> {
        if !parser_config.parse_items {
            return Ok(None);
        }
        let mut items = vec![];
        let base_path = resolver.get_behaviour_item_base();

        for file in WalkDir::new(resolver.get_behaviour_item_base())
            .into_iter()
            .filter_map(|e| {
                let e = e.ok();
                if let Some(e) = e {
                    if e.file_type().is_file() {
                        Some(e)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        {
            let data = std::fs::read_to_string(file.path())?;
            let raw_json = &serde_json::from_str::<Value>(&data)?;

            let item = Item::from_json(
                raw_json,
                parser_config
                    .item_register
                    .as_ref()
                    .unwrap_or(&FormattedComponentRegister::init_items()),
                Version::new(0, 0, 0),
            )?;

            if parser_config.skip_bland && item.is_bland() {
                continue;
            }

            resolver.item_path_lookup.insert(
                item.description.identifier.clone(),
                file.path()
                    .strip_prefix(&base_path)
                    .unwrap()
                    .to_path_buf()
                    .into_boxed_path(),
            );

            items.push(item);
        }

        Ok(Some(items))
    }
}
