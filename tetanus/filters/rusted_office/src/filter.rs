use crate::config::OfficeConfig;
use crate::stages::ast_stage::{ASTError, ASTStage};
use oxc::allocator::Allocator;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OfficeError {
    #[error(transparent)]
    ASTError(#[from] ASTError),
}

pub struct RustedOffice {
    pub config: OfficeConfig,
    pub alloc: Allocator,
}

impl RustedOffice {
    pub fn new(cfg: OfficeConfig) -> Self {
        Self {
            config: cfg.clone(),
            alloc: Default::default(),
        }
    }

    pub fn process(mut self) -> Result<(), OfficeError> {
        let ast = self.next_stage();
        let ripper = ast.next_state()?;

        Ok(())
    }

    fn next_stage(&mut self) -> ASTStage {
        ASTStage {
            config: self.config.clone(),
            alloc: &self.alloc,
        }
    }
}
