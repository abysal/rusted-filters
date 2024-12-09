use crate::addon::blocks::block::Block;
use crate::addon::items::item::Item;
use crate::addon::path_resolver::AddonPathResolver;
use crate::addon::traits::FormattedJsonSerialize;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use thiserror::Error;

pub struct Addon {
    pub resolver: Box<dyn AddonPathResolver>,
    blocks: HashMap<String, UnsafeCell<Block>>,
    items: HashMap<String, UnsafeCell<Item>>,
}

#[derive(Error, Debug)]
pub enum AddonSerError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl Addon {
    pub fn new<PathResolver: AddonPathResolver + 'static>(resolver: PathResolver) -> Self {
        Self {
            resolver: Box::from(resolver),
            blocks: HashMap::new(),
            items: HashMap::new(),
        }
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks
            .insert(block.description.identifier.clone(), UnsafeCell::new(block));
    }
    pub fn push_item(&mut self, block: Item) {
        self.items
            .insert(block.description.identifier.clone(), UnsafeCell::new(block));
    }

    pub fn blocks_ref(&self) -> &HashMap<String, UnsafeCell<Block>> {
        &self.blocks
    }

    pub fn blocks_mut_ref(&mut self) -> &mut HashMap<String, UnsafeCell<Block>> {
        &mut self.blocks
    }

    pub fn items_ref(&self) -> &HashMap<String, UnsafeCell<Item>> {
        &self.items
    }

    pub fn items_mut_ref(&mut self) -> &mut HashMap<String, UnsafeCell<Item>> {
        &mut self.items
    }

    pub fn write(&mut self) -> Result<(), AddonSerError> {
        for (id, block) in &mut self.blocks {
            let path = self.resolver.get_behaviour_block_output(id);
            std::fs::write(path, serde_json::to_string(&block.get_mut().to_json())?)?
        }

        for (id, item) in &mut self.items {
            let path = self.resolver.get_behaviour_item_output(id);
            std::fs::write(path, serde_json::to_string(&item.get_mut().to_json())?)?
        }

        Ok(())
    }
}
