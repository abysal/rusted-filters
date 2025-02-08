use crate::addon::blocks::block_component::BlockComponent;
use crate::addon::component::{Component, ComponentError, GenericComponent};
use crate::addon::traits::JsonSerialize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Default, Clone)]
pub struct MinecraftCustomComponents {
    pub component_ids: Vec<String>,
}

impl Component for MinecraftCustomComponents {
    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        "minecraft:custom_components"
    }

    fn static_new() -> Self
    where
        Self: Sized,
    {
        Default::default()
    }

    fn id(&self) -> &str {
        Self::static_id()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn comp_clone(&self) -> GenericComponent {
        Box::new(self.clone())
    }

    fn from_json_dynamic(
        &self,
        json: &Value,
        _id: &str,
    ) -> Result<GenericComponent, ComponentError> {
        Ok(Box::new(Self {
            component_ids: json
                .as_array()
                .ok_or(ComponentError::MemberNotType(Self::static_id(), "an array"))?
                .iter()
                .map(|e| {
                    e.as_str().and_then(|e| Some(e.to_string())).ok_or(
                        ComponentError::MissingMember(
                            "minecraft:custom_components member",
                            "a string",
                        ),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        }))
    }
}

impl JsonSerialize for MinecraftCustomComponents {
    fn to_json(&self) -> Value {
        Value::Array(
            self.component_ids
                .clone()
                .into_iter()
                .map(|e| Value::String(e))
                .collect(),
        )
    }
}

impl BlockComponent for MinecraftCustomComponents {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MinecraftDisplayNameItem {
    pub value: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MinecraftDisplayNameBlock(pub String);

impl JsonSerialize for MinecraftDisplayNameItem {
    fn to_json(&self) -> Value {
        serde_json::to_value(&self).expect("Failed to write Display Name!")
    }
}

impl Component for MinecraftDisplayNameItem {
    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        "minecraft:display_name"
    }

    fn static_new() -> Self
    where
        Self: Sized,
    {
        Default::default()
    }

    fn id(&self) -> &str {
        "minecraft:display_name"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn comp_clone(&self) -> GenericComponent {
        Box::from(self.clone())
    }

    fn from_json_dynamic(&self, json: &Value, _: &str) -> Result<GenericComponent, ComponentError> {
        Ok(Box::new(Self {
            value: json.as_str().and_then(|e| Some(e.to_string())).ok_or(
                ComponentError::MissingMember("minecraft:display_name member", "a string"),
            )?,
        }))
    }
}

impl JsonSerialize for MinecraftDisplayNameBlock {
    fn to_json(&self) -> Value {
        Value::String(self.0.clone())
    }
}

impl Component for MinecraftDisplayNameBlock {
    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        "minecraft:display_name"
    }

    fn static_new() -> Self
    where
        Self: Sized,
    {
        Default::default()
    }

    fn id(&self) -> &str {
        "minecraft:display_name"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn comp_clone(&self) -> GenericComponent {
        Box::from(self.clone())
    }

    fn from_json_dynamic(&self, json: &Value, _: &str) -> Result<GenericComponent, ComponentError> {
        Ok(Box::new(Self(
            json.as_str().and_then(|ele| Some(ele.to_string())).ok_or(
                ComponentError::MissingMember("minecraft:display_name member", "a string"),
            )?,
        )))
    }
}
