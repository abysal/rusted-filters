use crate::addon::component::Component;
pub trait BlockComponent: Component {}

pub type GenericBlockComponent = Box<dyn BlockComponent>;
