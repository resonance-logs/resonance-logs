pub mod commands;
pub mod extractor;
pub mod mappings;
/// Module extractor for parsing module data from SyncContainerData packets
/// and uploading to the module optimizer service.
///
/// This module extracts module information from the game's protobuf data,
/// following the logic from StarResonanceDps C# implementation.
pub mod types;
pub mod uploader;

pub use extractor::extract_modules;
pub use types::{ModuleInfo, ModulePart};
pub use uploader::upload_modules;
