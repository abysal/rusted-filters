use crate::addon::component::{ComponentError, FormattedComponentRegister};
use crate::addon::component_store::ComponentStore;
use crate::addon::menu_category::MenuCategory;
use crate::addon::traits::FormattedJsonSerialize;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemDescription {
    pub identifier: String,
    #[serde(default)]
    pub menu_category: MenuCategory,
    #[serde(default)]
    pub is_experimental: bool,
}

#[derive(Debug)]
pub struct Item {
    pub format_version: semver::Version,
    pub description: ItemDescription,
    pub components: ComponentStore,
}

impl Item {
    pub fn is_bland(&self) -> bool {
        !self.components.contains_non_minecraft()
    }
}

impl FormattedJsonSerialize for Item {
    type Error = ComponentError;

    fn to_json(&self) -> Value {
        json!({
            "format_version": self.format_version,
            "minecraft:item": {
                "description": self.description,
                "components": self.components.to_json()
            }
        })
    }

    fn from_json(
        json: &Value,
        register: &FormattedComponentRegister,
        _: Version,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let format: semver::Version = serde_json::from_value(
            json.get("format_version")
                .ok_or(ComponentError::MissingMember("format_version", "an item"))?
                .clone(),
        )?;

        let json = json
            .get("minecraft:item")
            .ok_or(ComponentError::MissingMember("minecraft:item", "an item"))?;

        let description = serde_json::from_value(
            json.get("description")
                .ok_or(ComponentError::MissingMember("description", "an item"))?
                .clone(),
        )?;

        let components = ComponentStore::from_json(
            json.get("components")
                .ok_or(ComponentError::MissingMember("components", "an item"))?,
            register,
            format.clone(),
        )?;

        Ok(Self {
            format_version: format,
            description,
            components,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::addon::component::{ComponentError, FormattedComponentRegister, UnknownComponent};
    use crate::addon::items::item::Item;
    use crate::addon::menu_category::Category;
    use crate::addon::traits::FormattedJsonSerialize;
    use semver::Version;
    use serde_json::json;

    #[test]
    fn item_de_test() -> Result<(), ComponentError> {
        let register = FormattedComponentRegister::new();
        let version = Version::new(0, 0, 0);
        let json = serde_json::from_value(
            json!({"format_version":"1.21.40","minecraft:item":{"description":{"identifier":"azur:bottle_star_moss","menu_category":{"category":"equipment","group":"itemGroup.name.potion","is_hidden_in_commands":false}},"components":{"minecraft:icon":"azur:bottle_star_moss","minecraft:max_stack_size":1,"azur:transformable_item":{"initial_item":"minecraft:glass_bottle","interact_on":"azur:star_moss_emitter","transform_block":"minecraft:flowing_water","consume_item":true,"sound":{"id":"bottle.fill"}},"azur:placeable":{"block":"azur:star_moss_emitter","consume_item":{"transform_item":"minecraft:glass_bottle"},"replace_blocks":["minecraft:water"],"sound":{"id":"bucket.empty_water"}}}}}),
        )?;

        let item = Item::from_json(&json, &register, version)?;

        // Test that the format_version matches
        assert_eq!(item.format_version, Version::parse("1.21.40").unwrap());

        // Test that the identifier is correct
        assert_eq!(item.description.identifier, "azur:bottle_star_moss");

        // Test that the menu category is correctly parsed
        assert_eq!(item.description.menu_category.category, Category::Equipment);
        assert_eq!(
            item.description.menu_category.group,
            "itemGroup.name.potion"
        );
        assert_eq!(item.description.menu_category.is_hidden_in_commands, false);

        // Test the components field
        let components = &item.components;

        // Verify the presence of "minecraft:icon" component
        let icon: Option<&UnknownComponent> = components.get_component_ref("minecraft:icon");
        assert!(icon.is_some());
        assert_eq!(icon.unwrap().data, json!("azur:bottle_star_moss"));

        // Verify the presence of "minecraft:max_stack_size" component
        let max_stack_size: Option<&UnknownComponent> =
            components.get_component_ref("minecraft:max_stack_size");
        assert!(max_stack_size.is_some());
        assert_eq!(max_stack_size.unwrap().data, json!(1));

        // Verify the presence of "azur:transformable_item" component
        let transformable_item: Option<&UnknownComponent> =
            components.get_component_ref("azur:transformable_item");
        assert!(transformable_item.is_some());
        let transformable_data = &transformable_item.unwrap().data;
        assert_eq!(transformable_data["initial_item"], "minecraft:glass_bottle");
        assert_eq!(transformable_data["interact_on"], "azur:star_moss_emitter");
        assert_eq!(
            transformable_data["transform_block"],
            "minecraft:flowing_water"
        );
        assert_eq!(transformable_data["consume_item"], true);
        assert_eq!(transformable_data["sound"]["id"], "bottle.fill");

        // Verify the presence of "azur:placeable" component
        let placeable: Option<&UnknownComponent> = components.get_component_ref("azur:placeable");
        assert!(placeable.is_some());
        let placeable_data = &placeable.unwrap().data;
        assert_eq!(placeable_data["block"], "azur:star_moss_emitter");
        assert_eq!(
            placeable_data["consume_item"]["transform_item"],
            "minecraft:glass_bottle"
        );
        assert_eq!(placeable_data["replace_blocks"], json!(["minecraft:water"]));
        assert_eq!(placeable_data["sound"]["id"], "bucket.empty_water");
        Ok(())
    }
}
