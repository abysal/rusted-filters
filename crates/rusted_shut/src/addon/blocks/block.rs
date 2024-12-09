use crate::addon::blocks::permutation::Permutation;
use crate::addon::component::{ComponentError, FormattedComponentRegister};
use crate::addon::component_store::ComponentStore;
use crate::addon::menu_category::MenuCategory;
use crate::addon::state::StateData;
use crate::addon::traits::FormattedJsonSerialize;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cell::UnsafeCell;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct PlacementDirection {
    pub enabled_states: Vec<String>,
    pub y_rotation_offset: i32,
}

impl Default for PlacementDirection {
    fn default() -> Self {
        Self {
            y_rotation_offset: 180,
            enabled_states: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PlacementPosition {
    pub enabled_states: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Trait {
    PlacementPosition(PlacementPosition),
    PlacementDirection(PlacementDirection),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockDescription {
    pub identifier: String,
    #[serde(default)]
    pub menu_category: MenuCategory,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub states: HashMap<String, StateData>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub traits: HashMap<String, Trait>,
}

impl TryInto<BlockDescription> for Value {
    type Error = serde_json::Error;
    fn try_into(self) -> Result<BlockDescription, Self::Error> {
        serde_json::from_value(self)
    }
}

#[derive(Debug)]
pub struct Block {
    pub description: BlockDescription,
    pub components: ComponentStore,
    pub permutations: Vec<Permutation>,
    pub format_version: semver::Version,
}

impl Block {
    pub fn is_bland(&self) -> bool {
        if self.components.contains_non_minecraft() {
            return false;
        }

        for p in &self.permutations {
            if p.components.contains_non_minecraft() {
                return false;
            }
        }
        true
    }
}

impl FormattedJsonSerialize for Block {
    type Error = ComponentError;

    fn to_json(&self) -> Value {
        json!({
            "format_version": self.format_version,
            "minecraft:block": {
                "description": self.description,
                "components": self.components.to_json(),
                "permutations": self.permutations.iter().map(|perm| perm.to_json()).collect::<Vec<Value>>()
            }
        })
    }

    fn from_json(
        json: &Value,
        register: &FormattedComponentRegister,
        _: semver::Version, // We actually create this in this call. So we just ignore it
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let format: semver::Version = serde_json::from_value(
            json.get("format_version")
                .ok_or(ComponentError::MissingMember("format_version", "a block"))?
                .clone(),
        )?;

        let json = json
            .get("minecraft:block")
            .ok_or(ComponentError::MissingMember("minecraft:block", "a block"))?;

        let description = serde_json::from_value(
            json.get("description")
                .ok_or(ComponentError::MissingMember("description", "a block"))?
                .clone(),
        )?;

        let components = ComponentStore::from_json(
            json.get("components")
                .ok_or(ComponentError::MissingMember("components", "a block"))?,
            register,
            format.clone(),
        )?;

        let permutations = if let Some(permutations) = json.get("permutations") {
            permutations
                .as_array()
                .ok_or(ComponentError::MemberNotType("permutations", "an array"))?
                .into_iter()
                .map(|ele| Permutation::from_json(ele, register, format.clone()))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        Ok(Self {
            format_version: format,
            description,
            components,
            permutations,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::addon::blocks::block::Block;
    use crate::addon::component::{ComponentError, FormattedComponentRegister, UnknownComponent};
    use crate::addon::state::{IntRange, StateData};
    use crate::addon::traits::FormattedJsonSerialize;
    use semver::Version;
    use serde_json::json;

    #[test]
    fn block_de_test() -> Result<(), ComponentError> {
        let register = FormattedComponentRegister::new();
        let ver = semver::Version::new(0, 0, 0);
        let json = json!({"format_version":"1.21.40","minecraft:block":{"description":{"identifier":"azur:star_moss_emitter","states":{"azur:growth":[0,1,2]}},"components":{"minecraft:material_instances":{"*":{"texture":"azur:nothing","render_method":"alpha_test"}},"minecraft:geometry":"minecraft:geometry.full_block","minecraft:collision_box":false,"minecraft:selection_box":{"origin":[-8,0,-8],"size":[16,16,16]},"minecraft:liquid_detection":{"detection_rules":[{"can_contain_liquid":true,"on_liquid_touches":"no_reaction"}]}},"permutations":[{"condition":"q.block_state('azur:growth') == 0","components":{"minecraft:tick":{"looping":true,"interval_range":[120,150]},"azur:particle_emitter":{"particleId":"azur:star_moss","molangVariables":[["growth",1]]}}},{"condition":"q.block_state('azur:growth') == 1","components":{"minecraft:tick":{"looping":true,"interval_range":[100,130]},"azur:particle_emitter":{"particleId":"azur:star_moss","molangVariables":[["growth",2]]}}},{"condition":"q.block_state('azur:growth') == 2","components":{"minecraft:tick":{"looping":true,"interval_range":[80,110]},"azur:particle_emitter":{"particleId":"azur:star_moss","molangVariables":[["growth",3]]}}}]}});

        let blk = Block::from_json(&json, &register, ver)?;

        // Verify the format version
        assert_eq!(blk.format_version, Version::parse("1.21.40").unwrap());

        // Verify the description identifier
        assert_eq!(blk.description.identifier, "azur:star_moss_emitter");

        // Verify the states in the description
        let growth_state = blk.description.states.get("azur:growth").unwrap();
        if let StateData::IntRange(IntRange(ref values)) = growth_state {
            assert_eq!(values, &vec![0, 1, 2]);
        } else {
            panic!("Expected IntRange for azur:growth state");
        }

        // Verify that there are no traits in the description
        assert!(blk.description.traits.is_empty());

        // Verify the components, ensuring the UnknownComponent structure is used
        let material_instances = blk
            .components
            .get_component_ref::<UnknownComponent>("minecraft:material_instances");
        assert!(material_instances.is_some());
        assert_eq!(
            material_instances.unwrap().id,
            "minecraft:material_instances"
        );

        let geometry = blk
            .components
            .get_component_ref::<UnknownComponent>("minecraft:geometry");
        assert!(geometry.is_some());
        assert_eq!(geometry.unwrap().id, "minecraft:geometry");

        let collision_box = blk
            .components
            .get_component_ref::<UnknownComponent>("minecraft:collision_box");
        assert!(collision_box.is_some());
        assert_eq!(collision_box.unwrap().id, "minecraft:collision_box");

        let selection_box = blk
            .components
            .get_component_ref::<UnknownComponent>("minecraft:selection_box");
        assert!(selection_box.is_some());
        assert_eq!(selection_box.unwrap().id, "minecraft:selection_box");

        let liquid_detection = blk
            .components
            .get_component_ref::<UnknownComponent>("minecraft:liquid_detection");
        assert!(liquid_detection.is_some());
        assert_eq!(liquid_detection.unwrap().id, "minecraft:liquid_detection");

        // Verify the size and condition of permutations
        assert_eq!(blk.permutations.len(), 3);

        // Check the first permutation condition and its components
        let first_perm = &blk.permutations[0];
        assert_eq!(first_perm.condition, "q.block_state('azur:growth') == 0");

        let tick_component_1 = first_perm
            .components
            .get_component_ref::<UnknownComponent>("minecraft:tick");
        assert!(tick_component_1.is_some());
        assert_eq!(tick_component_1.unwrap().id, "minecraft:tick");

        let particle_emitter_1 = first_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter");
        assert!(particle_emitter_1.is_some());
        assert_eq!(particle_emitter_1.unwrap().id, "azur:particle_emitter");

        // Check the second permutation condition and its components
        let second_perm = &blk.permutations[1];
        assert_eq!(second_perm.condition, "q.block_state('azur:growth') == 1");

        let tick_component_2 = second_perm
            .components
            .get_component_ref::<UnknownComponent>("minecraft:tick");
        assert!(tick_component_2.is_some());
        assert_eq!(tick_component_2.unwrap().id, "minecraft:tick");

        let particle_emitter_2 = second_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter");
        assert!(particle_emitter_2.is_some());
        assert_eq!(particle_emitter_2.unwrap().id, "azur:particle_emitter");

        // Check the third permutation condition and its components
        let third_perm = &blk.permutations[2];
        assert_eq!(third_perm.condition, "q.block_state('azur:growth') == 2");

        let tick_component_3 = third_perm
            .components
            .get_component_ref::<UnknownComponent>("minecraft:tick");
        assert!(tick_component_3.is_some());
        assert_eq!(tick_component_3.unwrap().id, "minecraft:tick");

        let particle_emitter_3 = third_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter");
        assert!(particle_emitter_3.is_some());
        assert_eq!(particle_emitter_3.unwrap().id, "azur:particle_emitter");

        let particle_1 = first_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter")
            .unwrap();
        if let Some(data) = particle_1.data.as_object() {
            let molang_variables = data.get("molangVariables").unwrap().as_array().unwrap();
            assert_eq!(molang_variables, &vec![json!(["growth", 1])]);
        }

        let particle_2 = second_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter")
            .unwrap();
        if let Some(data) = particle_2.data.as_object() {
            let molang_variables = data.get("molangVariables").unwrap().as_array().unwrap();
            assert_eq!(molang_variables, &vec![json!(["growth", 2])]);
        }

        let particle_3 = third_perm
            .components
            .get_component_ref::<UnknownComponent>("azur:particle_emitter")
            .unwrap();
        if let Some(data) = particle_3.data.as_object() {
            let molang_variables = data.get("molangVariables").unwrap().as_array().unwrap();
            assert_eq!(molang_variables, &vec![json!(["growth", 3])]);
        }
        Ok(())
    }
}
