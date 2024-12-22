use crate::addon::addon::Addon;
use crate::addon::component::UnknownComponent;
use crate::addon::custom_infrastructure::component::custom_block::{
    CustomBlockComponent, GenericBlockCustomComponent,
};
use crate::addon::custom_infrastructure::component::custom_item::{
    CustomItemComponent, GenericItemCustomComponent,
};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

pub struct AddonProcessor<BlockError, ItemError, UserState> {
    block_components: HashMap<String, GenericBlockCustomComponent<BlockError, UserState>>,
    item_components: HashMap<String, GenericItemCustomComponent<ItemError, UserState>>,
    user_state: UserState,
}

#[derive(Debug, Error)]
pub enum ProcessingError<BlockError: Debug, ItemError: Debug> {
    #[error(transparent)]
    BlockError(BlockError),
    #[error(transparent)]
    ItemError(ItemError),
    #[error("Custom component is an unexpected type")]
    ComponentInvalidType,
}

impl<BlockError: Debug, ItemError: Debug, UserState>
    AddonProcessor<BlockError, ItemError, UserState>
{
    pub fn new(state: UserState) -> Self {
        Self {
            block_components: HashMap::new(),
            item_components: HashMap::new(),
            user_state: state,
        }
    }

    pub fn state_mut(&mut self) -> &mut UserState {
        &mut self.user_state
    }

    pub fn state(&self) -> &UserState {
        &self.user_state
    }

    /// Binds a block component to the `AddonProcessor`.
    ///
    /// This method takes a block component and automatically uses its static ID for registration.
    ///
    /// # Arguments
    /// * `comp` - The block component implementing `CustomBlockComponent`.
    pub fn bind_block_component<
        T: CustomBlockComponent<Error = BlockError, UserState = UserState> + 'static,
    >(
        &mut self,
        comp: T,
    ) -> &mut Self {
        self.bind_block_component_name(comp, T::static_id())
    }

    /// Binds a block component to the `AddonProcessor` using a custom ID.
    ///
    /// # Arguments
    /// * `comp` - The block component implementing `CustomBlockComponent`.
    /// * `id` - The ID to associate with this component.
    pub fn bind_block_component_name<
        T: CustomBlockComponent<Error = BlockError, UserState = UserState> + 'static,
    >(
        &mut self,
        comp: T,
        id: &str,
    ) -> &mut Self {
        self.bind_block_component_box_name(Box::new(comp), id.into())
    }

    pub fn bind_block_component_box(
        &mut self,
        comp: Box<dyn CustomBlockComponent<Error = BlockError, UserState = UserState>>,
    ) -> &mut Self {
        self.block_components.insert(comp.id().to_string(), comp);
        self
    }

    /// Binds a boxed block component to the `AddonProcessor`.
    ///
    /// This method allows the direct use of boxed block components.
    ///
    /// # Arguments
    /// * `comp` - A boxed block component implementing `CustomBlockComponent`.
    /// * `id` - The ID to associate with this component.
    pub fn bind_block_component_box_name(
        &mut self,
        comp: Box<dyn CustomBlockComponent<Error = BlockError, UserState = UserState>>,
        id: String,
    ) -> &mut Self {
        self.block_components.insert(id, comp);
        self
    }

    /// Binds an item component to the `AddonProcessor`.
    ///
    /// This method takes an item component and automatically uses its static ID for registration.
    ///
    /// # Arguments
    /// * `comp` - The item component implementing `CustomItemComponent`.
    pub fn bind_item_component<
        T: CustomItemComponent<Error = ItemError, UserState = UserState> + 'static,
    >(
        &mut self,
        comp: T,
    ) -> &mut Self {
        self.bind_item_component_name(comp, T::static_id())
    }

    /// Binds an item component to the `AddonProcessor` using a custom ID.
    ///
    /// # Arguments
    /// * `comp` - The item component implementing `CustomItemComponent`.
    /// * `id` - The ID to associate with this component.
    pub fn bind_item_component_name<
        T: CustomItemComponent<Error = ItemError, UserState = UserState> + 'static,
    >(
        &mut self,
        comp: T,
        id: &str,
    ) -> &mut Self {
        self.bind_item_component_box_name(Box::new(comp), id.into())
    }

    pub fn bind_item_component_box(
        &mut self,
        comp: Box<dyn CustomItemComponent<Error = ItemError, UserState = UserState>>,
    ) -> &mut Self {
        self.item_components.insert(comp.id().to_string(), comp);
        self
    }

    /// Binds a boxed item component to the `AddonProcessor`.
    ///
    /// This method allows the direct use of boxed item components.
    ///
    /// # Arguments
    /// * `comp` - A boxed item component implementing `CustomItemComponent`.
    /// * `id` - The ID to associate with this component.
    pub fn bind_item_component_box_name(
        &mut self,
        comp: Box<dyn CustomItemComponent<Error = ItemError, UserState = UserState>>,
        id: String,
    ) -> &mut Self {
        self.item_components.insert(id, comp);
        self
    }

    /// ## Performs
    /// Applies the registered components to the addon
    /// ## Returns
    /// The addon passed in; if error user must handle.
    pub fn process_addon(
        &mut self,
        mut addon: Addon,
    ) -> Result<Addon, ProcessingError<BlockError, ItemError>> {
        addon = self
            .process_blocks(addon)
            .map_err(|err| ProcessingError::BlockError(err))?;

        addon = self
            .process_items(addon)
            .map_err(|err| ProcessingError::ItemError(err))?;

        Ok(addon)
    }

    fn process_blocks(&mut self, addon: Addon) -> Result<Addon, BlockError> {
        unsafe {
            let cell = UnsafeCell::new(addon);

            let iteration_mut = cell.get().as_mut().unwrap_unchecked();
            let pass_addon = cell.get().as_mut().unwrap_unchecked();

            for (_, blk) in iteration_mut.blocks_mut_ref().iter_mut() {
                // Processes basic components
                {
                    let mut component_iter = blk.components.clone();

                    for (component_id, information) in component_iter.non_minecraft_components_mut()
                    {
                        let mut component_ref = blk.components.clone();
                        if let Some(func) = self.block_components.get_mut(component_id) {
                            let base = information
                                .as_any()
                                .downcast_ref::<UnknownComponent>()
                                .unwrap();

                            func.apply_component(
                                &base.data,
                                blk,
                                &mut component_ref,
                                Some(pass_addon),
                                &mut self.user_state,
                            )?;
                            component_ref.remove_component(component_id);
                        }
                        blk.components = component_ref;
                    }
                }

                // Processes permutations
                {
                    let mut permutation_iter = blk.permutations.clone();

                    for (_idx, perm) in permutation_iter.iter_mut().enumerate() {
                        let mut components = perm.components.clone();
                        let mut component_ref = perm.components.clone();

                        for (component_id, information) in components.non_minecraft_components_mut()
                        {
                            if let Some(func) = self.block_components.get_mut(component_id) {
                                let base = information
                                    .as_any()
                                    .downcast_ref::<UnknownComponent>()
                                    .unwrap();

                                func.apply_component(
                                    &base.data,
                                    blk,
                                    &mut component_ref,
                                    Some(pass_addon),
                                    &mut self.user_state,
                                )?;
                                component_ref.remove_component(component_id);
                            }
                        }
                        perm.components = component_ref;
                    }
                    blk.permutations = permutation_iter
                }
            }

            Ok(cell.into_inner())
        }
    }

    fn process_items(&mut self, addon: Addon) -> Result<Addon, ItemError> {
        unsafe {
            let cell = UnsafeCell::new(addon);

            let iteration_mut = cell.get().as_mut().unwrap_unchecked();
            let pass_addon = cell.get().as_mut().unwrap_unchecked();

            for (id, item) in iteration_mut.items_mut_ref().iter_mut() {
                let mut pass_ref = item.components.clone();
                let mut components = item.components.clone();

                for (component_id, information) in components.non_minecraft_components_mut() {
                    if let Some(func) = self.item_components.get_mut(component_id) {
                        let base = information
                            .as_any()
                            .downcast_ref::<UnknownComponent>()
                            .unwrap();

                        func.apply_component(
                            &base.data,
                            item,
                            &mut pass_ref,
                            Some(pass_addon),
                            &mut self.user_state,
                        )?;
                        pass_ref.remove_component(id);
                    }
                }
                item.components = pass_ref;
            }
            Ok(cell.into_inner())
        }
    }

    /// Hands back `UserState` over to callee
    pub fn disband(self) -> UserState {
        self.user_state
    }
}
