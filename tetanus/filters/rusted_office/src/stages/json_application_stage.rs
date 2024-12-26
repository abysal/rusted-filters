use crate::component_registry::CustomComponentRegistry;
use rusted_shut::addon::addon::Addon;
use rusted_shut::addon::blocks::block::Block;
use rusted_shut::addon::component_store::ComponentStore;
use rusted_shut::addon::components::custom_components::MinecraftCustomComponents;
use rusted_shut::addon::custom_infrastructure::addon_processor::AddonProcessor;
use rusted_shut::addon::custom_infrastructure::component::custom_base::CustomComponent;
use rusted_shut::addon::custom_infrastructure::component::custom_block::{
    CustomBlockComponent, GenericBlockCustomComponent,
};
use rusted_shut::addon::custom_infrastructure::component::custom_item::{
    CustomItemComponent, GenericItemCustomComponent,
};
use rusted_shut::addon::items::item::Item;
use serde_json::Value;
use std::any::Any;
use std::hint::unreachable_unchecked;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockError {}
#[derive(Debug, Error)]
pub enum ItemError {}

pub struct OfficeState {
    pub registry: CustomComponentRegistry,
}

impl OfficeState {
    pub fn new(registry: CustomComponentRegistry) -> Self {
        Self { registry }
    }
}

pub type OfficeProcessor = AddonProcessor<BlockError, ItemError, OfficeState>;

#[derive(Clone)]
pub struct CustomComponentInstance {
    id: String,
}

impl CustomComponentInstance {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CustomComponent for CustomComponentInstance {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        unsafe { unreachable_unchecked() }
    }
}

impl CustomBlockComponent for CustomComponentInstance {
    type Error = BlockError;
    type UserState = OfficeState;
    fn block_clone(
        &self,
    ) -> Box<dyn CustomBlockComponent<Error = Self::Error, UserState = Self::UserState>> {
        Box::from(self.clone())
    }

    fn apply_component(
        &mut self,
        data: &serde_json::value::Value,
        owner: &mut Block,
        component_context: &mut ComponentStore,
        _: Option<&mut Addon>,
        state: &mut Self::UserState,
    ) -> Result<(), Self::Error> {
        let (r, skip) = state.registry.instance_component_block(
            self.id(),
            &owner.description.identifier,
            data.clone(),
        );

        if let Some(custom) = component_context
            .get_component_mut::<MinecraftCustomComponents>("minecraft:custom_components")
        {
            custom.component_ids.push(r);
        } else if !skip {
            let main_custom = owner
                .components
                .get_component_mut_default::<MinecraftCustomComponents>(
                    "minecraft:custom_components",
                )
                .unwrap();
            let mut new_custom = main_custom.clone();
            if let Some((idx, _)) = new_custom
                .component_ids
                .iter()
                .enumerate()
                .find(|(_idx, ele)| ele.starts_with(self.id()))
            {
                new_custom.component_ids[idx] = r;
            } else {
                new_custom.component_ids.push(r);
            }

            component_context.set_component(new_custom)
        }
        Ok(())
    }
}

impl CustomItemComponent for CustomComponentInstance {
    type UserState = OfficeState;
    type Error = ItemError;
    fn item_clone(
        &self,
    ) -> Box<dyn CustomItemComponent<Error = Self::Error, UserState = Self::UserState>> {
        Box::from(self.clone())
    }

    fn apply_component(
        &mut self,
        data: &Value,
        owner: &mut Item,
        component_context: &mut ComponentStore,
        _: Option<&mut Addon>,
        state: &mut Self::UserState,
    ) -> Result<(), Self::Error> {
        let (r, skip)  = state.registry.instance_component_item(
            self.id(),
            &owner.description.identifier,
            data.clone(),
        );

        if skip {
            return Ok(());
        }

        let custom = component_context
            .get_component_mut_default::<MinecraftCustomComponents>("minecraft:custom_components")
            .unwrap();
        custom.component_ids.push(r);
        Ok(())
    }
}

pub fn register_component_types(register: CustomComponentRegistry) -> OfficeProcessor {
    let mut block_components = Vec::new();
    let mut item_components = Vec::new();

    for r in register.block_list_iter() {
        block_components.push(Box::new(CustomComponentInstance::new(r.search_id.clone()))
            as GenericBlockCustomComponent<BlockError, OfficeState>)
    }

    for r in register.item_list_iter() {
        item_components.push(Box::new(CustomComponentInstance::new(r.search_id.clone()))
            as GenericItemCustomComponent<ItemError, OfficeState>);
    }

    let mut processor = OfficeProcessor::new(OfficeState::new(register));
    for c in block_components {
        processor.bind_block_component_box(c);
    }

    for c in item_components {
        processor.bind_item_component_box(c);
    }

    processor
}
