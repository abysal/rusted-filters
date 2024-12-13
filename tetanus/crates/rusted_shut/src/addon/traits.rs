use serde_json::Value;
use crate::addon::component::{FormattedComponentRegister};

pub trait BoxClone {
    fn box_clone(&self) -> Box<Self>;
}


impl<T: Clone> BoxClone for T {
    fn box_clone(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

pub trait JsonSerialize {
    fn to_json(&self) -> Value;
}

pub trait FormattedJsonSerialize {
    type Error;
    fn to_json(&self) -> Value;
    fn from_json(json: &Value, component_register: &FormattedComponentRegister, current_format: semver::Version) -> Result<Self, Self::Error>
    where Self: Sized;
}
