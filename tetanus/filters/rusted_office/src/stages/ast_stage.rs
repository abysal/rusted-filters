use crate::config::{OfficeConfig, ParserMode};
use crate::stages::component_rip_stage::ComponentRipStage;
use crate::stages::init_function_rip_stage::FunctionRipper;
use std::path::{Path, StripPrefixError};
use std::pin::Pin;
use swc_common::comments::SingleThreadedComments;
use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast::Module;
use swc_ecma_parser::{Parser, Syntax};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ASTError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("{0:?}")]
    ModuleError(swc_ecma_parser::error::Error),
    #[error(transparent)]
    StripPrefix(#[from] StripPrefixError),
}

pub struct ASTImpl {
    pub source: Pin<Box<String>>,
    pub module: Module,
    pub comments: SingleThreadedComments,
    pub relative_path: Box<Path>, // This is the path relative from the scripts base dir (this is needed for register generation)
}

pub struct ASTStage {
    pub config: OfficeConfig,
}

impl ASTStage {
    pub fn next_state(self) -> Result<FunctionRipper, ASTError> {
        let paths = walkdir::WalkDir::new(&self.config.script_search_location)
            .follow_links(true)
            .into_iter()
            .filter(|e| {
                let e = e.as_ref().ok();
                if let None = e {
                    return false;
                }

                let e = e.unwrap();
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .and_then(|e| {
                            if e.to_string_lossy()
                                == self.config.parsed_config.mode.clone().to_string()
                            {
                                Some(e)
                            } else {
                                None
                            }
                        })
                        .map_or(false, |_| true)
            })
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        let mut ast = Vec::new();

        for path in paths {
            let base: Box<Path> = Box::from(
                path.clone()
                    .into_path()
                    .strip_prefix(&self.config.script_search_location)?,
            );

            let source = Pin::new(Box::new(std::fs::read_to_string(path.into_path())?));

            let (module, comments) = self.emit_single_impl(&source)?;
            ast.push(ASTImpl {
                module,
                source,
                comments,
                relative_path: base,
            })
        }

        Ok(FunctionRipper::new(self.config, ast))
    }

    fn emit_single_impl(
        &self,
        source: &String,
    ) -> Result<(Module, SingleThreadedComments), ASTError> {
        let comments = SingleThreadedComments::default();

        let mut parser = Parser::new(
            if self.config.parsed_config.mode == ParserMode::TS {
                Syntax::Typescript(Default::default())
            } else {
                Syntax::Es(Default::default())
            },
            StringInput::new(source, BytePos(0), BytePos((source.len() - 1) as u32)),
            Some(&comments),
        );

        let module = parser
            .parse_program()
            .map_err(|e| ASTError::ModuleError(e))?
            .expect_module();

        Ok((module, comments))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::OfficeConfig;
    use crate::filter::RustedOffice;
    use std::path::PathBuf;

    #[test]
    fn comment_poll_test() {
        let cfg = OfficeConfig {
            parsed_config: Default::default(),
            script_search_location: PathBuf::new().into_boxed_path(),
        };

        let office = RustedOffice::new(cfg).force_ast();
        let source = "\
import {war} from \"snor\";
// This is a comment

// This is the comment i want
class Retro {constructor() {}}
        "
        .to_string();
        office.emit_single_impl(&source).unwrap();
    }
}
