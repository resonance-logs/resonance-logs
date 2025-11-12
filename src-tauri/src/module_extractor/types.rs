use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Represents a single module with its parts and attributes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModuleInfo {
    /// Unique identifier (from item.Value.Uuid)
    pub uuid: String,
    /// Module name (e.g., "基础攻击")
    pub name: String,
    /// Config ID (from item.Value.ConfigId)
    pub config_id: i32,
    /// Quality level (from item.Value.Quality)
    pub quality: i32,
    /// Category (ATTACK, DEFENSE, SUPPORT)
    pub category: String,
    /// List of module parts (attributes)
    pub parts: Vec<ModulePart>,
}

impl ModuleInfo {
    /// Calculate a content hash for this module (for deduplication)
    pub fn content_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.uuid.as_bytes());
        hasher.update(&self.config_id.to_le_bytes());
        hasher.update(&self.quality.to_le_bytes());
        for part in &self.parts {
            hasher.update(&part.part_id.to_le_bytes());
            hasher.update(&part.value.to_le_bytes());
        }
        hex::encode(hasher.finalize())
    }
}

/// Represents a single part/attribute of a module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModulePart {
    /// Part ID (from ModParts[i])
    pub part_id: i32,
    /// Part name (e.g., "力量加持")
    pub name: String,
    /// Part value (from InitLinkNums[i])
    pub value: i32,
    /// Part type ("basic" or "special")
    #[serde(rename = "type")]
    pub part_type: String,
}

/// Represents an unknown attribute ID encountered during extraction
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct UnknownAttribute {
    pub part_id: i32,
    pub first_seen: String,
    pub occurrence_count: u32,
    pub module_config_ids: Vec<i32>,
}

/// Request body for module import API
#[derive(Debug, Serialize)]
pub struct ImportModulesRequest {
    pub version: String,
    pub modules: Vec<ModuleImportData>,
}

/// Module data formatted for API import
#[derive(Debug, Serialize)]
pub struct ModuleImportData {
    pub uuid: String,
    pub name: String,
    pub config_id: i32,
    pub quality: i32,
    pub category: String,
    pub parts: Vec<PartImportData>,
}

/// Part data formatted for API import
#[derive(Debug, Serialize)]
pub struct PartImportData {
    pub part_id: i32,
    pub name: String,
    pub value: i32,
    #[serde(rename = "type")]
    pub part_type: String,
}

/// Response from module import API
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct ImportModulesResponse {
    pub summary: ImportSummary,
    #[serde(default)]
    pub errors: Vec<ImportError>,
}

/// Summary of import operation
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct ImportSummary {
    pub added: i32,
    pub updated: i32,
    pub errors: i32,
}

/// Error for a specific module during import
#[derive(Debug, Serialize, Deserialize, specta::Type)]
pub struct ImportError {
    pub index: i32,
    pub uuid: String,
    pub error: String,
}

impl From<ModuleInfo> for ModuleImportData {
    fn from(module: ModuleInfo) -> Self {
        Self {
            uuid: module.uuid,
            name: module.name,
            config_id: module.config_id,
            quality: module.quality,
            category: module.category,
            parts: module.parts.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<ModulePart> for PartImportData {
    fn from(part: ModulePart) -> Self {
        Self {
            part_id: part.part_id,
            name: part.name,
            value: part.value,
            part_type: part.part_type,
        }
    }
}
