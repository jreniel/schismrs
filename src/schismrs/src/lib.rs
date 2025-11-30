// schismrs/src/lib.rs

// pub mod cache;
pub mod cli;
pub mod config;
// pub mod error;
// pub mod orchestrator;
pub mod constants;
pub mod state;
pub mod sync;

// Re-export commonly used types
// pub use cache::CacheManager;
pub use config::ModelConfig;
// pub use error::{Result, SchismError};
// pub use orchestrator::Orchestrator;
// pub use state::ProjectState;
// pub use sync::{ChangeDetector, ChangeSet, SchismGroup};
