use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::common::{EntityId, Timestamp};

/// Licenses the platform recognizes on content modules.
///
/// `Unlicensed` exists for homebrew that hasn't been released under
/// any license; it is valid for user-published modules but never for
/// the pre-bundled content shipped with the app and server, which must
/// carry one of the supported licenses.
#[typeshare]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseKind {
    #[serde(rename = "cc-by-4.0")]
    CcBy40,
    #[serde(rename = "ogl-1.0a")]
    Ogl10a,
    #[serde(rename = "unlicensed")]
    Unlicensed,
}

impl LicenseKind {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::CcBy40 => "CC-BY-4.0",
            Self::Ogl10a => "OGL 1.0a",
            Self::Unlicensed => "Unlicensed",
        }
    }

    /// The serialized wire/storage value (matches the serde renames).
    pub fn wire_value(self) -> &'static str {
        match self {
            Self::CcBy40 => "cc-by-4.0",
            Self::Ogl10a => "ogl-1.0a",
            Self::Unlicensed => "unlicensed",
        }
    }

    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "cc-by-4.0" => Some(Self::CcBy40),
            "ogl-1.0a" => Some(Self::Ogl10a),
            "unlicensed" => Some(Self::Unlicensed),
            _ => None,
        }
    }

    pub fn is_supported_for_bundling(self) -> bool {
        !matches!(self, Self::Unlicensed)
    }

    /// Maps an Open5e license key to the corresponding kind.
    pub fn from_open5e_key(key: &str) -> Option<Self> {
        match key {
            "cc-by-40" => Some(Self::CcBy40),
            "ogl-10a" => Some(Self::Ogl10a),
            _ => None,
        }
    }
}

/// A content pack (source book, supplement, or homebrew set).
///
/// Every other entity references this through `content_module_uuid` so
/// that catalog filtering, licensing, and per-pack restrictions can be
/// applied uniformly. The `schema_version` field captures which
/// authoring-schema the bundle was produced against — independent of
/// the publication date — and lets the importer detect packs that
/// pre-date a structural change.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentModule {
    pub uuid: EntityId,
    pub name: String,
    pub slug: String,
    pub license: LicenseKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_url: Option<String>,
    /// Schema version this module was authored against. Defaults to the
    /// crate's `SCHEMA_VERSION` when omitted.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub ordering: i32,
    /// Semantic version of the module's content (distinct from
    /// `schema_version`, which versions the authoring schema). Bumped
    /// each time a Setting publishes an updated module via the
    /// Promote-to-Module wizard.
    #[serde(default = "default_version_string")]
    pub version_string: String,
    /// Points to the previous version of this logical module, if any.
    /// The chain of `previous_version_uuid` links forms the module's
    /// version history; consumers can pin or upgrade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_version_uuid: Option<EntityId>,
    /// Set when the module was created via the Promote-to-Module
    /// wizard. Distinct from `created_at` because draft modules (used
    /// as setting-scoped staging) exist before they're published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

fn default_schema_version() -> u32 {
    crate::version::SCHEMA_VERSION
}

fn default_true() -> bool {
    true
}

fn default_version_string() -> String {
    "1.0.0".to_string()
}
