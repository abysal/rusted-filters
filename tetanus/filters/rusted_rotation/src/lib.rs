#![allow(deprecated)]
use rusted_shut::addon::addon::Addon;
use rusted_shut::addon::blocks::block::{Block, PlacementDirection, Trait};
use rusted_shut::addon::blocks::permutation::Permutation;
use rusted_shut::addon::component::{Component, UnknownComponent};
use rusted_shut::addon::component_store::ComponentStore;
use rusted_shut::addon::custom_infrastructure::component::custom_base::CustomComponent;
use rusted_shut::addon::custom_infrastructure::component::custom_block::{
    CustomBlockComponent, EmptyBlockState,
};
use serde::Deserialize;
use serde_json::json;
use std::any::Any;
use std::collections::HashMap;
use strum_macros::{Display, EnumString, ToString};

pub struct Rotation;
#[derive(Deserialize)]
struct RotationConfig {
    #[serde(default)]
    y_rotation: bool,
}

#[derive(ToString, Eq, PartialEq)]
enum Mode {
    #[strum(serialize = "minecraft:cardinal_direction")]
    Cardinal,
    #[strum(serialize = "minecraft:facing_direction")]
    Facing,
}
impl CustomComponent for Rotation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn id(&self) -> &str {
        Self::static_id()
    }

    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        "azur:rotation"
    }
}

#[allow(non_camel_case_types)]
#[derive(Display, EnumString)]
enum Dir {
    north,
    east,
    south,
    west,
    up,
    down,
}

struct TranslationInformation {
    dir: Dir,
    transform: [i32; 3],
}

const SHARED: [TranslationInformation; 4] = [
    TranslationInformation {
        transform: [0, 0, 0],
        dir: Dir::north,
    },
    TranslationInformation {
        transform: [0, 90, 0],
        dir: Dir::west,
    },
    TranslationInformation {
        transform: [0, 180, 0],
        dir: Dir::south,
    },
    TranslationInformation {
        transform: [0, -90, 0],
        dir: Dir::east,
    },
];

const UP_DOWN: [TranslationInformation; 2] = [
    TranslationInformation {
        transform: [90, 0, 0],
        dir: Dir::up,
    },
    TranslationInformation {
        transform: [-90, 0, 0],
        dir: Dir::down,
    },
];

impl Rotation {
    fn bind_perms(blk: &mut Block, add_y: bool) {
        let mode = if add_y { Mode::Facing } else { Mode::Cardinal };

        for info in SHARED {
            blk.permutations.push(Permutation::new(
                format!("q.block_state('{}') == '{}'", mode.to_string(), info.dir),
                ComponentStore::from_map(HashMap::from([(
                    "minecraft:transformation".to_string(),
                    Box::new(UnknownComponent::new(
                        json!({"rotation": info.transform}),
                        "minecraft:transformation".to_string(),
                    )) as Box<dyn Component>,
                )])),
            ));
        }

        if mode == Mode::Cardinal {
            return;
        }

        for info in UP_DOWN {
            blk.permutations.push(Permutation::new(
                format!("q.block_state('{}') == '{}'", mode.to_string(), info.dir),
                ComponentStore::from_map(HashMap::from([(
                    "minecraft:transformation".to_string(),
                    Box::new(UnknownComponent::new(
                        json!({"rotation": info.transform}),
                        "minecraft:transformation".to_string(),
                    )) as Box<dyn Component>,
                )])),
            ));
        }
    }
}

impl CustomBlockComponent for Rotation {
    type UserState = EmptyBlockState;
    type Error = serde_json::Error;
    fn block_clone(
        &self,
    ) -> Box<dyn CustomBlockComponent<Error = Self::Error, UserState = Self::UserState>> {
        Box::new(Self)
    }

    fn apply_component(
        &mut self,
        data: &serde_json::value::Value,
        owner: &mut Block,
        _: &mut ComponentStore,
        _: Option<&mut Addon>,
        _: &mut Self::UserState,
    ) -> Result<(), Self::Error> {
        let config = serde_json::from_value::<RotationConfig>(data.clone())?;
        Self::bind_perms(owner, config.y_rotation);
        let mode = if config.y_rotation {
            Mode::Facing
        } else {
            Mode::Cardinal
        };

        let add = Trait::PlacementDirection(PlacementDirection {
            enabled_states: vec![mode.to_string()],
            y_rotation_offset: 180,
        });

        owner
            .description
            .traits
            .insert("minecraft:placement_direction".to_string(), add);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Rotation;
    use rusted_shut::addon::addon::Addon;
    use rusted_shut::addon::blocks::block::Block;
    use rusted_shut::addon::component::FormattedComponentRegister;
    use rusted_shut::addon::custom_infrastructure::addon_processor::AddonProcessor;
    use rusted_shut::addon::custom_infrastructure::component::custom_block::EmptyBlockState;
    use rusted_shut::addon::path_resolver::default_impl::BaseResolver;
    use rusted_shut::addon::traits::FormattedJsonSerialize;
    use serde_json::json;

    #[test]
    fn rotation_test() -> Result<(), serde_json::Error> {
        let rot = Rotation;
        let json = json!({
          "format_version": "1.21.40",
          "minecraft:block": {
            "description": {
              "identifier": "azur:sea_shells",
              "menu_category": {
                "category": "none",
                "is_hidden_in_commands": true
              },
              "states": {
                "azur:sea_shell_state": [0, 1, 2, 3]
              }
            },
            "components": {
              "minecraft:material_instances": {
                "*": {
                  "texture": "azur:sea_shells_texture",
                  "render_method": "alpha_test_single_sided"
                }
              },
              "minecraft:geometry": {
                "identifier": "geometry.azur.seashells",
                "bone_visibility": {
                  "2": "q.block_state('azur:sea_shell_state') >= 1",
                  "3": "q.block_state('azur:sea_shell_state') >= 2",
                  "4": "q.block_state('azur:sea_shell_state') == 3"
                }
              },
              "azur:rotation": {},
            }
          }
        });

        let blk = Block::from_json(
            &json,
            &FormattedComponentRegister::new(),
            semver::Version::new(0, 0, 0),
        )
        .unwrap();

        let mut addon = Addon::new(BaseResolver::new("./".into()));
        addon.push_block(blk);

        let mut process =
            AddonProcessor::<serde_json::Error, serde_json::Error, EmptyBlockState>::new(
                EmptyBlockState,
            );
        process.bind_block_component(rot);

        let _ = process.process_addon(addon).unwrap();
        Ok(())
    }
}
