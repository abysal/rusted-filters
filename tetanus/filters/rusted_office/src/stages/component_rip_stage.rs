use crate::component_registry::{
    ComponentInformation, ComponentInformationError, ComponentStaticInformation,
    CustomComponentRegistry,
};
use crate::config::OfficeConfig;
use crate::find_n_with_comments;
use crate::stages::ast_stage::ASTImpl;
use crate::string_extensions::*;
use log::error;
use swc_common::comments::{Comment, Comments};
use swc_common::{BytePos, Spanned};
use swc_ecma_ast::{ClassDecl, ExportDecl, Function};
use swc_ecma_visit::{
    AstNodePath, AstParentNodeRef, Visit, VisitAstPath, VisitWith, VisitWithAstPath,
};
use thiserror::Error;

const COMPONENT_START_IDENTIFIER: &'static str = "@Generate";
const COMPONENT_START_IDENTIFIER_CONSUME: &'static str = "@Generate(";

#[derive(Debug, Error)]
pub enum ComponentRipperError {
    #[error("argument: {0}, doesn't follow the passing scheme of \"arg_id: arg_value\"")]
    InvalidArgumentFormat(String),
    #[error(transparent)]
    InformationParseError(#[from] ComponentInformationError),
}

pub struct ComponentRipStage {
    config: OfficeConfig,
    source_list: Vec<ASTImpl>,
}

impl ComponentRipStage {
    pub fn new(config: OfficeConfig, source_list: Vec<ASTImpl>) -> Self {
        Self {
            config,
            source_list,
        }
    }

    pub fn build_registry(self) -> Result<Option<CustomComponentRegistry>, ComponentRipperError> {
        let mut out = Vec::new();
        for ast in &self.source_list {
            if let Some(ci) = self.process_file(ast) {
                for i in ci {
                    out.push(i)
                }
            }
        }

        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CustomComponentRegistry::build_from_list(out)))
        }
    }

    fn process_file(&self, ast: &ASTImpl) -> Option<Vec<ComponentInformation>> {
        let classes = Self::pull_classes_with_comments(ast)?;

        let mut return_information = Vec::new();

        for (class, comments) in classes {
            let comp_ci = {
                let _class_info = class.class.as_ref();

                let comment = Self::consolidate_comments(comments);

                let parse_location = Self::contains_generate_expression(&comment)?;

                let result = comment
                    .slice_extended(ByteIndex(parse_location), EndIndex)
                    .scoped_slice(&[
                        ScopeDeclaration {
                            begin: '{',
                            end: '}',
                        },
                        ScopeDeclaration {
                            begin: '[',
                            end: ']',
                        },
                        ScopeDeclaration {
                            begin: '(',
                            end: ')',
                        },
                    ])?;

                let args = match Self::build_argument_list(result) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!(
                            "Failed to rip component class {}, with error {}",
                            class.ident.sym.as_str(),
                            e
                        );
                        continue;
                    }
                };

                let info = match ComponentInformation::new(
                    args,
                    ComponentStaticInformation::builder()
                        .class_id(class.ident.sym.as_str().to_string())
                        .relative_path(ast.relative_path.clone())
                        .build(),
                ) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                };
                Some(info)
            };

            if let Some(i) = comp_ci {
                return_information.push(i);
            }
        }

        if return_information.is_empty() {
            None
        } else {
            Some(return_information)
        }
    }

    fn build_argument_list(whole_comment: &str) -> Result<Vec<(&str, &str)>, ComponentRipperError> {
        let mut arg_count = Vec::new();
        Self::pull_single_arg(whole_comment, &mut arg_count)?;
        Ok(arg_count)
    }

    fn pull_single_arg<'a>(
        comment_section: &'a str,
        out: &mut Vec<(&'a str, &'a str)>,
    ) -> Result<(), ComponentRipperError> {
        let (current, next) = comment_section.scoped_split(
            ',',
            &[
                ScopeDeclaration {
                    begin: '{',
                    end: '}',
                },
                ScopeDeclaration {
                    begin: '[',
                    end: ']',
                },
                ScopeDeclaration {
                    begin: '(',
                    end: ')',
                },
            ],
            false,
            1,
        );

        let value = current
            .split_once(':')
            .ok_or(ComponentRipperError::InvalidArgumentFormat(
                current.to_string(),
            ))?;
        out.push((value.0.trim(), value.1.trim()));

        if let Some(val) = next {
            Self::pull_single_arg(val.trim(), out)
        } else {
            Ok(())
        }
    }

    // TODO: Make this use the source map and not do a useless clone
    fn consolidate_comments(comments: Vec<Comment>) -> String {
        comments
            .into_iter()
            .map(|ele| format!("{} ", ele.text.to_string()))
            .collect()
    }

    fn contains_generate_expression(str: &str) -> Option<usize> {
        str.find(COMPONENT_START_IDENTIFIER)
            .and_then(|ele| Option::from(ele + COMPONENT_START_IDENTIFIER.as_bytes().len()))
    }

    fn pull_classes_with_comments(ast: &ASTImpl) -> Option<Vec<(ClassDecl, Vec<Comment>)>> {
        find_n_with_comments!(class, ast)
    }
}
