use crate::addon::traits::JsonSerialize;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("{0} is not {1}")]
    MemberNotType(&'static str, &'static str),
    #[error("{0} is not {1}")]
    MemberNotTypeDynamic(String, &'static str),

    #[error("{0} is not an object")]
    NotObject(&'static str),
    #[error("{0} is not an object")]
    NotObjectDynamic(String),

    #[error("{0} is missing from {1}!")]
    MissingMember(&'static str, &'static str),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

pub trait Component: Any + JsonSerialize + Debug {
    fn static_id() -> &'static str
    where
        Self: Sized;
    fn static_new() -> Self
    where
        Self: Sized;

    fn id(&self) -> &str;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn comp_clone(&self) -> GenericComponent;

    fn from_json_dynamic(&self, json: &Value, id: &str) -> GenericComponent;
}

pub type UsedComponent = dyn Component;

pub type GenericComponent = Box<dyn Component>;

impl Clone for GenericComponent {
    fn clone(&self) -> Self {
        self.comp_clone()
    }
}

#[derive(Clone, Debug)]
pub enum VersionRestriction {
    Min(semver::Version),
    Max(semver::Version),
    MinMax(semver::Version, semver::Version),
}

impl VersionRestriction {
    pub fn contains(&self, version: &semver::Version) -> bool {
        match self {
            VersionRestriction::Min(min) => version >= min,
            VersionRestriction::Max(max) => version <= max,
            VersionRestriction::MinMax(min, max) => version >= min && version <= max,
        }
    }
}

pub struct FormattedComponentRegister {
    internal: HashMap<String, Vec<(VersionRestriction, GenericComponent)>>,
}

impl FormattedComponentRegister {
    pub fn new() -> Self {
        Self {
            internal: Default::default(),
        }
    }

    pub fn bind_component<T: Component>(&mut self, ver: VersionRestriction) {
        if let Some(v) = self.internal.get_mut(T::static_id()) {
            v.push((ver, Box::new(T::static_new())));
        } else {
            self.internal.insert(
                T::static_id().to_string(),
                vec![(ver, Box::new(T::static_new()))],
            );
        }
    }

    pub fn get_component(&self, format: &semver::Version, id: &str) -> Option<&GenericComponent> {
        let comp = self.internal.get(id)?;

        comp.iter().find_map(|(ver, comp)| {
            if ver.contains(format) {
                Some(comp)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct UnknownComponent {
    pub data: Value,
    pub id: String,
}

impl UnknownComponent {
    pub fn new(json: Value, id: String) -> Self {
        Self { id, data: json }
    }
}

impl JsonSerialize for UnknownComponent {
    fn to_json(&self) -> Value {
        self.data.clone()
    }
}

impl Component for UnknownComponent {
    fn static_id() -> &'static str
    where
        Self: Sized,
    {
        unimplemented!("This isn't possible to implement with unknown data")
    }

    fn static_new() -> Self
    where
        Self: Sized,
    {
        Self {
            id: "".into(),
            data: Default::default(),
        }
    }

    fn id(&self) -> &str {
        &self.id
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

    fn from_json_dynamic(&self, json: &Value, id: &str) -> GenericComponent {
        Box::new(Self {
            id: id.into(),
            data: json.clone(),
        })
    }
}
