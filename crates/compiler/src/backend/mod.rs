use crate::lir;

pub mod rs;
pub mod ts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendKind {
    TypeScript,
    Rust,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodegenTarget {
    TypeScript,
    TypeScriptDefinition,
    JavaScript,
    Rust,
    Python,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    UnsupportedTarget(CodegenTarget),
    EmitFailed(String),
}

pub trait Backend: Send + Sync {
    fn kind(&self) -> BackendKind;
    fn supports(&self, target: CodegenTarget) -> bool;
    fn emit(&self, file: &lir::File, target: CodegenTarget) -> Result<String, BackendError>;
}

pub struct BackendRegistry {
    backends: Vec<Box<dyn Backend>>,
}

impl BackendRegistry {
    pub fn with_defaults() -> Self {
        Self {
            backends: vec![
                Box::new(ts::TypeScriptBackend::new()),
                Box::new(rs::RustBackend::new()),
            ],
        }
    }

    pub fn register<B>(&mut self, backend: B)
    where
        B: Backend + 'static,
    {
        self.backends.push(Box::new(backend));
    }

    pub fn emit(&self, file: &lir::File, target: CodegenTarget) -> Result<String, BackendError> {
        let backend = self
            .backends
            .iter()
            .find(|backend| backend.supports(target))
            .ok_or(BackendError::UnsupportedTarget(target))?;
        backend.emit(file, target)
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

pub fn emit(file: &lir::File, target: CodegenTarget) -> Result<String, BackendError> {
    BackendRegistry::with_defaults().emit(file, target)
}
