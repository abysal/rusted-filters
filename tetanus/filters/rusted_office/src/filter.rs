use crate::config::OfficeConfig;
use crate::stages::ast_stage::{ASTError, ASTStage};
use crate::stages::component_rip_stage::ComponentRipperError;
use crate::stages::emitter::CodeEmitter;
use crate::stages::json_application_stage::{register_component_types, BlockError, ItemError};
use rusted_shut::addon::addon::Addon;
use rusted_shut::addon::custom_infrastructure::addon_processor::ProcessingError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OfficeError {
    #[error(transparent)]
    ASTError(#[from] ASTError),
    #[error(transparent)]
    RIPError(#[from] ComponentRipperError),
    #[error(transparent)]
    ProcessingError(#[from] ProcessingError<BlockError, ItemError>),
    #[error(transparent)]
    FileError(#[from] std::io::Error),
}

pub struct RustedOffice {
    pub config: OfficeConfig,
}

impl RustedOffice {
    pub fn new(cfg: OfficeConfig) -> Self {
        Self {
            config: cfg.clone(),
        }
    }

    pub fn process(self, addon: Addon) -> Result<Addon, OfficeError> {
        let ast = self.next_stage();

        let func_ripper = ast.next_state()?;
        let (funcs, ripper) = func_ripper.next_stage();
        let registry = ripper.build_registry()?;
        if let None = registry {
            return Ok(addon);
        }

        let mut processor = register_component_types(registry.unwrap());
        let addon = processor.process_addon(addon)?;
        let registry = processor.disband().registry;

        let emitter = CodeEmitter::new(registry, funcs);

        let source = emitter.emit();

        std::fs::write(
            {
                let mut pth = PathBuf::from(self.config.script_search_location.clone());
                pth.push("rusted_office_register.ts");
                pth
            },
            source,
        )?;

        std::fs::write(
            {
                let mut pth = PathBuf::from(self.config.script_search_location.clone());
                pth.push(self.config.script_entry.clone());
                pth
            },
            format!(
                "import {{initOfficeComponents}} from \"./rusted_office_register\"; import {{world as MyCustomRegister}} from \"@minecraft/server\";MyCustomRegister.beforeEvents.worldInitialize.subscribe(initOfficeComponents); \n{}",
                std::fs::read_to_string({
                    let mut pth = PathBuf::from(self.config.script_search_location);
                    pth.push(self.config.script_entry);
                    pth
                })?
            ),
        )?;

        Ok(addon)
    }

    fn next_stage(&self) -> ASTStage {
        ASTStage {
            config: self.config.clone(),
        }
    }

    #[cfg(test)]
    pub fn force_ast(&self) -> ASTStage {
        self.next_stage()
    }
}
