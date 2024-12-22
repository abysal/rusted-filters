use crate::addon::component::{ComponentError, FormattedComponentRegister};
use crate::addon::component_store::ComponentStore;
use crate::addon::traits::FormattedJsonSerialize;
use semver::Version;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct Permutation {
    pub condition: String,
    pub components: ComponentStore,
}

impl Permutation {
    pub fn new(condition: String, components: ComponentStore) -> Self {
        Self {
            condition,
            components,
        }
    }
}

impl FormattedJsonSerialize for Permutation {
    type Error = ComponentError;

    fn to_json(&self) -> Value {
        json!({
            "condition": self.condition,
            "components": self.components.to_json()
        })
    }

    fn from_json(
        json: &Value,
        formatted_component_register: &FormattedComponentRegister,
        version: Version,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let condition = json
            .get("condition")
            .ok_or(ComponentError::MissingMember("condition", "permutation"))?
            .as_str()
            .ok_or(ComponentError::MemberNotType("condition", "a string"))?
            .to_string();

        let components = ComponentStore::from_json(
            json.get("components")
                .ok_or(ComponentError::MissingMember("components", "permutation"))?,
            formatted_component_register,
            version,
        )?;

        Ok(Self {
            condition,
            components,
        })
    }
}
