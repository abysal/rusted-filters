use crate::addon::component::{
    Component, ComponentError, FormattedComponentRegister, GenericComponent, UnknownComponent,
};
use crate::addon::traits::FormattedJsonSerialize;
use serde_json::{Map, Value};
use std::collections::btree_map::{Iter, IterMut};
use std::collections::BTreeMap;
use std::iter::Filter;

#[derive(Default, Debug)]
pub struct ComponentStore {
    components: BTreeMap<String, GenericComponent>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_component<T: Component + Clone>(&self, name: &str) -> Option<T> {
        if let Some(c) = self.components.get(name)?.as_any().downcast_ref::<T>() {
            Some(c.clone())
        } else {
            None
        }
    }

    pub fn get_component_ref<T: Component>(&self, name: &str) -> Option<&T> {
        if let Some(c) = self.components.get(name)?.as_any().downcast_ref::<T>() {
            Some(c)
        } else {
            None
        }
    }

    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(T::static_id())?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    pub fn remove_component(&mut self, name: &str) {
        self.components.remove(name);
    }

    pub fn set_component<T: Component>(&mut self, comp: T) {
        self.set_component_custom(comp, T::static_id().to_string())
    }

    pub fn set_component_custom<T: Component>(&mut self, comp: T, name: String) {
        self.set_component_box(Box::new(comp), name)
    }

    pub fn set_component_box(&mut self, comp: Box<dyn Component>, name: String) {
        self.components.insert(name, comp);
    }

    pub fn contains_non_minecraft(&self) -> bool {
        for _ in self
            .components
            .iter()
            .filter(|(id, _)| !id.starts_with("minecraft:"))
        {
            return true;
        }
        false
    }

    pub fn non_minecraft_components(
        &self,
    ) -> Filter<Iter<String, GenericComponent>, fn(&(&String, &GenericComponent)) -> bool> {
        self.components
            .iter()
            .filter(|(id, _)| !id.starts_with("minecraft:"))
    }

    pub fn non_minecraft_components_mut(
        &mut self,
    ) -> Filter<IterMut<String, GenericComponent>, fn(&(&String, &mut GenericComponent)) -> bool>
    {
        self.components
            .iter_mut()
            .filter(|(id, _)| !id.starts_with("minecraft:"))
    }
}

impl FormattedJsonSerialize for ComponentStore {
    type Error = ComponentError;

    fn to_json(&self) -> Value {
        Value::from(Map::from_iter(
            self.components
                .iter()
                .map(|(id, comp)| (id.clone(), comp.to_json()))
                .into_iter(),
        ))
    }

    fn from_json(
        json: &Value,
        formatted_component_register: &FormattedComponentRegister,
        current_format: semver::Version,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let obj = json
            .as_object()
            .ok_or(ComponentError::NotObject("components"))?;
        let components = obj
            .iter()
            .map(|(id, blob)| {
                if let Some(comp) = formatted_component_register.get_component(&current_format, id)
                {
                    (id.clone(), comp.from_json_dynamic(blob, id))
                } else {
                    (
                        id.clone(),
                        UnknownComponent::static_new().from_json_dynamic(blob, id),
                    )
                }
            })
            .collect();

        Ok(Self { components })
    }
}
