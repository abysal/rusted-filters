use crate::addon::addon::Addon;
use crate::addon::blocks::block::Block;
use crate::addon::component_store::ComponentStore;
use crate::addon::custom_infrastructure::component::custom_base::CustomComponent;
use serde_json::Value;

pub struct EmptyBlockState;

pub trait CustomBlockComponent: CustomComponent {
    type UserState = EmptyBlockState;

    fn clone(
        &self,
    ) -> Box<dyn CustomBlockComponent<Error = Self::Error, UserState = Self::UserState>>;

    fn apply_component(
        &mut self,
        owner: &mut Block,
        component_context: &mut ComponentStore,
        owning_addon: Option<&mut Addon>,
        state: &mut Self::UserState,
    ) -> Result<(), Self::Error>;

    fn from_json_dynamic(
        &self,
        json: &Value,
        state: &mut Self::UserState,
    ) -> Box<dyn CustomBlockComponent<Error = Self::Error, UserState = Self::UserState>>;
}

pub type GenericBlockCustomComponent<BlockError, UserState> =
    Box<dyn CustomBlockComponent<Error = BlockError, UserState = UserState>>;
