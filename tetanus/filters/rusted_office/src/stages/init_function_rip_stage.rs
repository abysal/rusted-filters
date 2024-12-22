use crate::config::OfficeConfig;
use crate::find_n_with_comments;
use crate::stages::ast_stage::ASTImpl;
use crate::stages::component_rip_stage::ComponentRipStage;
use bon::Builder;
use std::path::Path;
use swc_common::comments::Comment;
use swc_ecma_ast::FnDecl;

#[derive(Builder, Debug)]
pub struct FunctionInformation {
    pub function_name: String,
    pub path: Box<Path>,
}

pub struct FunctionRipper {
    ast_info: Vec<ASTImpl>,
    config: OfficeConfig,
}

const INIT_FUNC_STRING: &'static str = "@InitFunction";

impl FunctionRipper {
    pub fn new(config: OfficeConfig, ast_info: Vec<ASTImpl>) -> Self {
        Self { ast_info, config }
    }

    pub fn next_stage(self) -> (Vec<FunctionInformation>, ComponentRipStage) {
        let mut out = Vec::new();

        for ast in &self.ast_info {
            let functions: Vec<_> = {
                let funcs = find_n_with_comments!(func, ast);
                if let Some(funcs) = funcs {
                    funcs
                        .into_iter()
                        .filter_map(|(func, comments)| {
                            let mut valid = false;
                            for c in comments {
                                if c.text.as_str().contains(INIT_FUNC_STRING) {
                                    valid = true;
                                    break;
                                }
                            }

                            if !valid {
                                return None;
                            }

                            Some(
                                FunctionInformation::builder()
                                    .function_name(func.ident.sym.to_string())
                                    .path(ast.relative_path.clone())
                                    .build(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            };

            out.extend(functions.into_iter());
        }

        (out, ComponentRipStage::new(self.config, self.ast_info))
    }
}
