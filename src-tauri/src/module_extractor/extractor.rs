/// Module extraction logic ported from C# StarResonanceDps ModuleParser.cs
/// Extracts module information from SyncContainerData protobuf packets

use crate::module_extractor::mappings;
use crate::module_extractor::types::{ModuleInfo, ModulePart};
use blueprotobuf_lib::blueprotobuf::SyncContainerData;
use log::{debug, warn};

/// Extract modules from SyncContainerData packet
///
/// This is equivalent to the C# ParseModuleInfo function.
/// Processes CharSerialize data to extract module inventory.
///
/// # Arguments
/// * `sync_data` - Decoded SyncContainerData protobuf message
///
/// # Returns
/// Vec of ModuleInfo structs, empty if no modules found or parsing fails
pub fn extract_modules(sync_data: &SyncContainerData) -> Vec<ModuleInfo> {
    let mut modules = Vec::new();

    // Get VData (CharSerialize) from the sync packet
    let v_data = match &sync_data.v_data {
        Some(data) => data,
        None => {
            debug!("No VData in SyncContainerData");
            return modules;
        }
    };

    // Get Mod.ModInfos - contains attribute values for each module
    let mod_infos = match &v_data.r#mod {
        Some(mod_data) => &mod_data.mod_infos,
        None => {
            debug!("No Mod data in CharSerialize");
            return modules;
        }
    };

    // Get ItemPackage.Packages - contains all inventory items
    let item_package = match &v_data.item_package {
        Some(pkg) => pkg,
        None => {
            debug!("No ItemPackage in CharSerialize");
            return modules;
        }
    };

    // Iterate through all packages (backpack containers)
    for (_package_type, package) in &item_package.packages {
        // Iterate through items in this package
        for (item_key, item_value) in &package.items {
            // Check if this item has ModNewAttr with ModParts - indicates it's a module
            let mod_new_attr = match &item_value.mod_new_attr {
                Some(attr) if !attr.mod_parts.is_empty() => attr,
                _ => continue, // Not a module, skip
            };

            let config_id = match item_value.config_id {
                Some(id) => id,
                None => {
                    warn!("Item missing config_id");
                    continue;
                }
            };
            let uuid = match item_value.uuid {
                Some(id) => id,
                None => {
                    warn!("Item missing uuid");
                    continue;
                }
            };
            let quality = match item_value.quality {
                Some(q) => q,
                None => {
                    warn!("Item missing quality");
                    continue;
                }
            };

            // Get module name from config ID
            let module_name = match mappings::get_module_name(config_id) {
                Some(name) => name,
                None => {
                    warn!("Unknown module config_id: {}", config_id);
                    continue;
                }
            };

            // Get module category
            let category = match mappings::get_module_category(config_id) {
                Some(cat) => cat,
                None => {
                    warn!("Unknown module category for config_id: {}", config_id);
                    continue;
                }
            };

            // Get the ModParts (attribute IDs) from ModNewAttr
            let mod_parts = &mod_new_attr.mod_parts;

            // Get the InitLinkNums (attribute values) from ModInfos
            let init_link_nums = match mod_infos.get(item_key) {
                Some(info) => &info.init_link_nums,
                None => {
                    warn!("No ModInfo found for item key: {}", item_key);
                    continue;
                }
            };

            // Build the parts list (min of parts count and values count)
            let parts_count = std::cmp::min(mod_parts.len(), init_link_nums.len());
            let mut parts = Vec::with_capacity(parts_count);

            for i in 0..parts_count {
                let part_id = mod_parts[i];
                let part_value = init_link_nums[i];

                // Get attribute name
                let attr_name = match mappings::get_attr_name(part_id) {
                    Some(name) => name,
                    None => {
                        warn!("Unknown attribute ID: {}", part_id);
                        &format!("未知属性({})", part_id)
                    }
                };

                // Get attribute type (basic or special)
                let attr_type = mappings::get_attr_type(part_id);

                parts.push(ModulePart {
                    part_id,
                    name: attr_name.to_string(),
                    value: part_value,
                    part_type: attr_type.to_string(),
                });
            }

            // Create ModuleInfo
            modules.push(ModuleInfo {
                uuid: uuid.to_string(),
                name: module_name.to_string(),
                config_id,
                quality,
                category: category.to_string(),
                parts,
            });
        }
    }

    debug!("Extracted {} modules from SyncContainerData", modules.len());
    modules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_modules_empty_data() {
        let sync_data = SyncContainerData { v_data: None };
        let modules = extract_modules(&sync_data);
        assert_eq!(modules.len(), 0);
    }

    // Additional tests would require mock protobuf data
}
