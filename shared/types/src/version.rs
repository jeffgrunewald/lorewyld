use serde::{Deserialize, Serialize};

/// Current Lorewyld content-bundle schema version.
///
/// Bumped whenever any schema in this crate gains, removes, or renames a
/// required field. Importers compare a bundle's declared version against
/// this constant before attempting to load records.
pub const SCHEMA_VERSION: u32 = 2;

/// Minimum schema version this build can still import. Bundles authored
/// at any version in `MIN_SUPPORTED_SCHEMA_VERSION..=SCHEMA_VERSION` are
/// accepted; older ones must be re-exported by a newer authoring tool.
pub const MIN_SUPPORTED_SCHEMA_VERSION: u32 = 2;

/// Envelope metadata exposed at the top of every `ContentBundle`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    /// Schema version the bundle was authored against.
    pub version: u32,
    /// Minimum schema version the bundle expects an importer to support.
    /// Equal to `version` for forward-only schemas.
    pub min_supported: u32,
}

impl SchemaVersion {
    pub const fn current() -> Self {
        Self {
            version: SCHEMA_VERSION,
            min_supported: MIN_SUPPORTED_SCHEMA_VERSION,
        }
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}
