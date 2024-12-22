use bon::Builder;
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use std::slice::Iter;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentInformationError {
    #[error("Invalid Component Type Specified: {0}")]
    InvalidComponentType(String),
    #[error("Missing Required Param: {0}, type: {1}, class: {2}")]
    MissingRequiredParam(&'static str, &'static str, String),
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ComponentType {
    Item,
    Block,
    Both,
    Invalid,
}

impl FromStr for ComponentType {
    type Err = ComponentInformationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "item" => Ok(Self::Item),
            "block" => Ok(Self::Block),
            "both" => Ok(Self::Both),
            _ => Err(ComponentInformationError::InvalidComponentType(
                s.to_string(),
            )),
        }
    }
}

#[derive(Builder, Debug, Clone)]
pub struct ComponentStaticInformation {
    pub class_id: String,
    pub relative_path: Box<Path>,
}

#[derive(Debug, Clone)]
pub struct ComponentInformation {
    pub search_id: String,
    pub component_type: ComponentType,
    pub is_pure_data: bool,
    pub pass_id: bool,
    pub information: ComponentStaticInformation,
}

impl ComponentInformation {
    pub fn new(
        arg_list: Vec<(&str, &str)>,
        information: ComponentStaticInformation,
    ) -> Result<Self, ComponentInformationError> {
        let mut self_data = Self {
            search_id: "".to_string(),
            component_type: ComponentType::Invalid,
            is_pure_data: false,
            pass_id: false,
            information,
        };

        for (name, val) in arg_list {
            match name {
                "id" => self_data.search_id = val.to_string(),
                "type" => self_data.component_type = val.parse()?,
                "pureData" => self_data.is_pure_data = bool::from_str(val).unwrap_or(false),
                "passId" => self_data.pass_id = bool::from_str(val).unwrap_or(false),
                v => {
                    log::warn!("Unknown param: name: {}, value: {}", v, val)
                }
            }
        }

        if self_data.search_id.is_empty() {
            return Err(ComponentInformationError::MissingRequiredParam(
                "id",
                "string",
                self_data.information.class_id.clone(),
            ));
        }
        if self_data.component_type == ComponentType::Invalid {
            return Err(ComponentInformationError::MissingRequiredParam(
                "type",
                "component type",
                self_data.information.class_id.clone(),
            ));
        }

        Ok(self_data)
    }
}

pub struct ComponentInstance {
    pub static_information: Rc<ComponentInformation>,
    pub owner_id: Option<String>,
    pub instance_id: usize,
    pub data: serde_json::value::Value,
}

#[derive(Default)]
pub struct CustomComponentRegistry {
    block_list: HashMap<String, Rc<ComponentInformation>>,
    item_list: HashMap<String, Rc<ComponentInformation>>,

    instance_id: usize,

    block_instances: Vec<ComponentInstance>,
    item_instances: Vec<ComponentInstance>,
}

impl CustomComponentRegistry {
    pub fn build_from_list(components: Vec<ComponentInformation>) -> Self {
        let mut self_data: CustomComponentRegistry = Default::default();
        for component in components {
            match component.component_type {
                ComponentType::Item => {
                    self_data
                        .item_list
                        .insert(component.search_id.clone(), Rc::from(component));
                }
                ComponentType::Block => {
                    self_data
                        .block_list
                        .insert(component.search_id.clone(), Rc::from(component));
                }
                ComponentType::Both => {
                    self_data
                        .item_list
                        .insert(component.search_id.clone(), Rc::from(component.clone()));
                    self_data
                        .block_list
                        .insert(component.search_id.clone(), Rc::from(component));
                }
                ComponentType::Invalid => unreachable!(),
            }
        }

        self_data
    }

    pub fn block_list_iter(&self) -> Values<'_, String, Rc<ComponentInformation>> {
        self.block_list.values()
    }

    pub fn item_list_iter(&self) -> Values<'_, String, Rc<ComponentInformation>> {
        self.item_list.values()
    }

    pub fn block_instances_iter(&self) -> Iter<'_, ComponentInstance> {
        self.block_instances.iter()
    }

    pub fn item_instances_iter(&self) -> Iter<'_, ComponentInstance> {
        self.item_instances.iter()
    }

    pub fn instance_component_block(
        &mut self,
        component_id: &str,
        owner_id: &str,
        data: serde_json::value::Value,
    ) -> String {
        let blk = &self.block_list;
        if let Some(comp) = blk.get(component_id) {
            self.instance_id += 1;

            self.block_instances.push(ComponentInstance {
                static_information: comp.clone(),
                owner_id: if comp.pass_id {
                    Some(owner_id.to_string())
                } else {
                    None
                },
                instance_id: self.instance_id,
                data: data.clone(),
            });
            let ret = format!("{}_{}", component_id, self.instance_id);
            ret
        } else {
            panic!("Component Instance Not Valid")
        }
    }

    pub fn instance_component_item(
        &mut self,
        component_id: &str,
        owner_id: &str,
        data: serde_json::value::Value,
    ) -> String {
        let blk = &self.item_list;
        if let Some(comp) = blk.get(component_id) {
            self.instance_id += 1;

            self.item_instances.push(ComponentInstance {
                static_information: comp.clone(),
                owner_id: if comp.pass_id {
                    Some(owner_id.to_string())
                } else {
                    None
                },
                instance_id: self.instance_id,
                data: data.clone(),
            });
            let ret = format!("{}_{}", component_id, self.instance_id);
            ret
        } else {
            panic!("Component Instance Not Valid")
        }
    }
}
