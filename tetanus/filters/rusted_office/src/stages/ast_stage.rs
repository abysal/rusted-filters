use crate::config::OfficeConfig;
use crate::stages::component_rip_stage::ComponentRipStage;
use oxc::allocator::Allocator;
use oxc::ast::ast::Program;
use oxc::parser::{ParseOptions, Parser};
use thiserror::Error;

pub type OxcBox<'a, T> = oxc::allocator::Box<'a, T>;

#[derive(Debug, Error)]
pub enum ASTError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub struct ASTImpl<'a> {
    pub prog: Program<'a>,
    pub alloc: &'a Allocator,
}

pub struct ASTStage<'a> {
    pub alloc: &'a Allocator,
    pub config: OfficeConfig,
}

impl<'a> ASTStage<'a> {
    pub fn next_state(mut self) -> Result<ComponentRipStage<'a>, ASTError> {
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
            let mut opt = ParseOptions::default();
            opt.parse_regular_expression = false;

            let str = Box::new(std::fs::read_to_string(path.path())?);
            ast.push(ASTImpl {
                alloc: self.alloc,
                prog: Parser::new(
                    self.alloc,
                    str.leak(),
                    self.config.parsed_config.mode.clone().into(),
                )
                .parse()
                .program,
            });
        }

        Ok(ComponentRipStage::new(self.config, self.alloc, ast))
    }
}
