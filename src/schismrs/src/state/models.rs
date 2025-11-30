// schismrs/src/state/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root state structure for the entire project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub project: ProjectInfo,

    /// Fingerprints of each generator's last successful sync
    /// Key: generator state_key (e.g., "param", "bctides")
    /// Value: config fingerprint at time of generation
    #[serde(default)]
    pub generator_fingerprints: HashMap<String, GeneratorState>,

    /// Hashes of source files that generators depend on
    /// Key: source file identifier (e.g., "hgrid", "vgrid")
    /// Value: content hash and metadata
    #[serde(default)]
    pub source_hashes: HashMap<String, SourceFileState>,
}

/// Basic project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub initialized_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

/// State of a generator after last sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorState {
    /// Fingerprint of config sections this generator depends on
    pub fingerprint: String,

    /// When this generator was last successfully run
    pub synced_at: DateTime<Utc>,
}

/// State of a source file that generators depend on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileState {
    /// Content hash of the source file
    pub hash: String,

    /// Relative path to the source file
    pub path: PathBuf,

    /// When this hash was last computed
    pub checked_at: DateTime<Utc>,
}
