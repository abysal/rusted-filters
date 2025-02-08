use crate::config::{OfficeConfig, ParserMode};
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

            match self.emit_single_impl(&source) {
                Ok(r) => {
                    if let Some((module, comments)) = r {
                        ast.push(ASTImpl {
                            module,
                            source,
                            comments,
                            relative_path: base,
                        })
                    }
                }
                Err(e) => {
                    log::error!("Failed to parse: {:?}, with error: {:?}", base, e);
                }
            }
        }

        Ok(FunctionRipper::new(self.config, ast))
    }

    fn emit_single_impl(
        &self,
        source: &String,
    ) -> Result<Option<(Module, SingleThreadedComments)>, ASTError> {
        let comments = SingleThreadedComments::default();

        if source.is_empty() {
            return Ok(None);
        }

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
            .parse_module()
            .map_err(|e| ASTError::ModuleError(e))?;

        Ok(Some((module, comments)))
    }
}
