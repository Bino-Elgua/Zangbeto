pub mod event;
pub mod ir;
pub mod drift;
pub mod replay;
pub mod ledger;
pub mod pipeline;

#[derive(thiserror::Error, Debug)]
pub enum ZangbetoError {
    #[error("Ledger error: {0}")]
    Ledger(String),
    #[error("Execution error: {0}")]
    Execution(String),
    #[error("Bridge error: {0}")]
    Bridge(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
