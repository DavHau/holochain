//! Collection of cells to form a holochain application
use crate::cell::CellId;
use holo_hash::{AgentPubKey, DnaHash};
use holochain_serialized_bytes::SerializedBytes;
use std::{collections::HashMap, path::PathBuf};

/// Placeholder used to identify apps
pub type AppId = String;

/// A collection of [DnaHash]es paired with an [AgentPubKey] and an app id
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AppPaths {
    /// Placeholder to find the app
    pub app_id: AppId,
    /// The agent that installed this app
    pub agent_key: AgentPubKey,
    /// The Dna paths in this app
    pub dnas: Vec<(PathBuf, Option<serde_json::Value>)>,
}

/// App storage
pub type Apps = HashMap<AppId, Vec<CellId>>;

#[derive(Clone, Debug)]
/// A collection of [CellIds]s paired with an app id
pub struct App {
    /// Placeholder to find the app
    pub app_id: AppId,
    /// Cells in this app
    pub cell_ids: Vec<CellId>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// All the proofs that are needed to complete genesis on an app
pub struct MembraneProofs {
    /// A map of [DnaHash] to proofs
    pub proofs: HashMap<DnaHash, SerializedBytes>,
}

impl MembraneProofs {
    /// Create an empty set of membrane proofs
    pub fn empty() -> Self {
        Self {
            proofs: HashMap::new(),
        }
    }
}
