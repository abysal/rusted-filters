use crate::addon::blocks::block::Block;
use crate::addon::items::item::Item;
use crate::addon::path_resolver::AddonPathResolver;
use std::cell::UnsafeCell;
use std::collections::HashMap;

pub struct Addon {
    pub resolver: Box<dyn AddonPathResolver>,
    blocks: HashMap<String, UnsafeCell<Block>>,
    items: HashMap<String, UnsafeCell<Item>>,
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
}
