use crate::addon::blocks::block::Block;
use crate::addon::items::item::Item;

pub struct TranslationManager;

impl TranslationManager {
    pub fn key_for_block(block: &Block) -> String {
        format!("block.{}", block.description.identifier)
    }

    pub fn key_for_item(item: &Item) -> String {
        format!("item.{}", item.description.identifier)
    }
}
