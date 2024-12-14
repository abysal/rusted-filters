use crate::config::OfficeConfig;
use crate::stages::ast_stage::ASTImpl;
use oxc::allocator::Allocator;
use oxc::ast::ast::Statement;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComponentRipperError {}

pub struct ComponentRipStage<'a> {
    config: OfficeConfig,
    alloc: &'a Allocator,
    source_list: Vec<ASTImpl<'a>>,
}

impl<'a> ComponentRipStage<'a> {
    pub fn new(config: OfficeConfig, alloc: &'a Allocator, source_list: Vec<ASTImpl<'a>>) -> Self {
        Self {
            config,
            alloc,
            source_list,
        }
    }

    pub fn next_state(self) -> Result<(), ComponentRipperError> {
        Ok(())
    }

    fn process_file(&self, ast: &ASTImpl<'a>) -> () {
        for r in &ast.prog.body {
            let class = match r {
                Statement::ClassDeclaration(v) => v,
                _ => continue,
            };
            class.span.start
        }
    }
}
