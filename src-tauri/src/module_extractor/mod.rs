/// Module extractor for parsing module data from SyncContainerData packets
/// and uploading to the module optimizer service.
///
/// This module extracts module information from the game's protobuf data,
/// following the logic from StarResonanceDps C# implementation.

pub mod types;
pub mod mappings;
pub mod extractor;
pub mod uploader;
pub mod commands;

pub use extractor::extract_modules;
pub use uploader::upload_modules;
pub use types::{ModuleInfo, ModulePart};
