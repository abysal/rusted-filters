use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
    pub format_version: i32,
    pub header: Header,
    modules: Option<Vec<Module>>,
    subpacks: Option<Vec<Subpack>>,
    dependencies: Option<Vec<Dependency>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Header {
    pub description: String,
    pub name: String,
    pub uuid: String, // UUID as string
    pub version: Version,
}

impl Manifest {
    pub fn add_module(&mut self, module: Module) {
        if let Some(modules) = &mut self.modules {
            modules.push(module);
        } else {
            self.modules = Some(vec![module]);
        }
    }

    pub fn add_subpack(&mut self, subpack: Subpack) {
        if let Some(subpacks) = &mut self.subpacks {
            subpacks.push(subpack);
        } else {
            self.subpacks = Some(vec![subpack]);
        }
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        if let Some(dependencies) = &mut self.dependencies {
            dependencies.push(dependency);
        } else {
            self.dependencies = Some(vec![dependency]);
        }
    }

    pub fn get_modules(&mut self) -> Option<&mut Vec<Module>> {
        self.modules.as_mut()
    }

    pub fn get_subpack(&mut self) -> Option<&mut Vec<Subpack>> {
        self.subpacks.as_mut()
    }

    pub fn get_dependency(&mut self) -> Option<&mut Vec<Dependency>> {
        self.dependencies.as_mut()
    }
}


#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Module {
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    pub uuid: String, // UUID as string
    pub version: Version,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleType {
    Resources,
    Data,
    ClientData,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Subpack {
    pub folder_name: String,
    pub name: String,
    pub memory_tier: i32,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Dependency {
    pub uuid: String, // UUID as string
    pub version: Version,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Version {
    #[serde(default)]
    #[serde(rename = "type")]
    pub version_type: Option<i32>,
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub minor_version: i32,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::addon::mainifest::Manifest;

    #[test]
    fn parse_manifest() -> Result<(), serde_json::Error> {
        let json = json!({
          "format_version": 2,
          "header": {
            "name": "Azuryth",
            "description": "By theaddonn",
            "uuid": "134be210-dfe8-4714-8fa7-2857c8a30116",
            "version": [1, 0, 0],
            "min_engine_version": [1, 21, 20]
          },
          "modules": [
            {
              "type": "data",
              "uuid": "e3741960-a214-493b-be75-67d91999a07a",
              "version": [1, 0, 0]
            }
          ],
          "dependencies": [
            {
              "uuid": "c4ba1ec1-ec16-44a2-b0ee-864fc2058895",
              "version": [1, 0, 0]
            }
          ]
        }
        );

        let manifest= serde_json::to_value(serde_json::from_value::<Manifest>(json.clone())?)?;

        assert_eq!(manifest, json);

        Ok(())
    }
}
