use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category{
    Construction,
    Equipment,
    Items,
    Nature,
    None
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MenuCategory {
    pub category: Category,
    #[serde(default)]
    pub is_hidden_in_commands: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub group: String
}


impl Default for MenuCategory {
    fn default() -> Self {
        Self {
            category: Category::None,
            is_hidden_in_commands: false,
            group: String::new()
        }
    }
}