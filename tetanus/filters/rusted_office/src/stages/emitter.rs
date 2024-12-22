use crate::component_registry::{ComponentInformation, ComponentInstance, CustomComponentRegistry};
use crate::stages::init_function_rip_stage::FunctionInformation;
use std::fmt::Write;

pub struct CodeEmitter {
    registry: CustomComponentRegistry,
    functions: Vec<FunctionInformation>,
}

impl CodeEmitter {
    pub fn new(registry: CustomComponentRegistry, functions: Vec<FunctionInformation>) -> Self {
        Self {
            registry,
            functions,
        }
    }

    pub fn emit(&self) -> String {
        let mut out = String::new();

        self.import(&mut out);

        self.function(&mut out);

        out
    }

    fn function(&self, out: &mut String) {
        let emit_body = || {
            let mut result = String::new();

            for f in &self.functions {
                writeln!(&mut result, "{}();", f.function_name).expect("Failed to write");
            }

            let mut emit_call_expression =
                |instance: &ComponentInstance, registry_name: &'static str| {
                    let call_expression = format!(
                        "new {}({}{})",
                        instance.static_information.information.class_id,
                        serde_json::to_string(&instance.data)
                            .expect("Failed to process data to json"),
                        if instance.static_information.pass_id {
                            format!(
                                ", \"{}\"",
                                instance
                                    .owner_id
                                    .as_ref()
                                    .expect("Owner ID not set on pass id")
                            )
                        } else {
                            "".to_string()
                        }
                    );

                    writeln!(&mut result,
                             "try {{ {} }} catch (e) {{ console.error(`Major Error When registering component: {}, subid: {}, error: ${{e}}`);}}",
                             if instance.static_information.is_pure_data {
                                format!("{};", call_expression)
                            } else {
                                format!(
                                    "event.{}.registerCustomComponent(\"{}\", {});",
                                    registry_name,
                                    format!(
                                        "{}_{}",
                                        instance.static_information.search_id, instance.instance_id
                                    ),
                                    call_expression
                                )
                            },
                            instance.static_information.information.class_id,
                             format!(
                                 "{}_{}",
                                 instance.static_information.search_id, instance.instance_id
                             ),
                    ).expect("Failed to write");
                };

            for instance in self.registry.block_instances_iter() {
                emit_call_expression(instance, "blockComponentRegistry");
            }

            for instance in self.registry.item_instances_iter() {
                emit_call_expression(instance, "itemComponentRegistry");
            }

            result
        };

        let body = emit_body();
        writeln!(
            out,
            "export function initOfficeComponents(event: WorldInitializeBeforeEvent) {{ {} }}",
            body
        )
        .expect("Failed to write");
    }

    fn import(&self, out: &mut String) {
        writeln!(
            out,
            "import {{WorldInitializeBeforeEvent}} from \"@minecraft/server\";"
        )
        .expect("Failed to write");

        let path = |ci: &ComponentInformation| {
            let mut str = ci
                .information
                .relative_path
                .to_string_lossy()
                .replace("\\", "/");
            let len = str.len();
            str.truncate(len - 3);
            str
        };

        for ci in self.registry.block_list_iter() {
            writeln!(
                out,
                "import {{ {} }} from \"./{}\";",
                ci.information.class_id,
                path(ci)
            )
            .expect("Failed to write");
        }

        for ii in self.registry.item_list_iter() {
            writeln!(
                out,
                "import {{ {} }} from \"./{}\";",
                ii.information.class_id,
                path(ii)
            )
            .expect("Failed to write");
        }

        for fi in &self.functions {
            writeln!(out, "import {{ {} }} from \"./{}\";", fi.function_name, {
                let mut str = fi.path.to_string_lossy().replace("\\", "/");
                let len = str.len();
                str.truncate(len - 3);
                str
            })
            .expect("Failed to write");
        }
    }
}
