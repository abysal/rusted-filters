use crate::addon::addon::Addon;
use crate::addon::component_store::ComponentStore;
use crate::addon::custom_infrastructure::component::custom_base::CustomComponent;
use crate::addon::items::item::Item;
use serde_json::Value;

pub struct EmptyBlockState;

pub trait CustomItemComponent: CustomComponent {
    type UserState = EmptyBlockState;
    fn clone(
        &self,
    ) -> Box<dyn CustomItemComponent<Error = Self::Error, UserState = Self::UserState>>;
    fn apply_component(
        &mut self,
        owner: &mut Item,
        component_context: &mut ComponentStore,
        owning_addon: Option<&mut Addon>,
        state: &mut Self::UserState,
    ) -> Result<(), Self::Error>;
    fn from_json_dynamic(
        &self,
        json: &Value,
        state: &mut Self::UserState,
    ) -> Box<dyn CustomItemComponent<Error = Self::Error, UserState = Self::UserState>>;
}

pub type GenericItemCustomComponent<BlockError, UserState> =
    Box<dyn CustomItemComponent<Error = BlockError, UserState = UserState>>;
