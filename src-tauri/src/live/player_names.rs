use std::collections::HashMap;

/// Database-backed player name management
pub struct PlayerNames;

impl PlayerNames {
    /// Get player name by UID from database
    pub fn get_name_by_uid(uid: i64) -> Option<String> {
        match crate::database::commands::get_name_by_uid(uid) {
            Ok(name) => name,
            Err(e) => {
                log::error!("Failed to get name for UID {}: {}", uid, e);
                None
            }
        }
    }

    /// Get recent players ordered by last seen (most recent first)
    pub fn get_recent_players(limit: usize) -> Vec<(i64, String)> {
        match crate::database::commands::get_recent_players(limit as i64) {
            Ok(players) => players,
            Err(e) => {
                log::error!("Failed to get recent players: {}", e);
                Vec::new()
            }
        }
    }

    /// Get multiple names by UIDs (batch query for performance)
    pub fn get_names_by_uids(uids: &[i64]) -> HashMap<i64, String> {
        let mut result = HashMap::new();
        
        for &uid in uids {
            if let Some(name) = Self::get_name_by_uid(uid) {
                result.insert(uid, name);
            }
        }
        
        result
    }

    /// Check if a player exists in the database
    pub fn contains_player(uid: i64) -> bool {
        Self::get_name_by_uid(uid).is_some()
    }
}