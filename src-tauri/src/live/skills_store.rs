use crate::live::commands_models::SkillsWindow;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SkillsStore {
    dps_skills: HashMap<i64, SkillsWindow>,
    heal_skills: HashMap<i64, SkillsWindow>,
    active_subscriptions: HashSet<(i64, String)>,
}

impl SkillsStore {
    pub fn new() -> Self {
        Self {
            dps_skills: HashMap::new(),
            heal_skills: HashMap::new(),
            active_subscriptions: HashSet::new(),
        }
    }

    pub fn update_dps_skills(&mut self, player_uid: i64, skills_window: SkillsWindow) {
        self.dps_skills.insert(player_uid, skills_window);
    }

    pub fn update_heal_skills(&mut self, player_uid: i64, skills_window: SkillsWindow) {
        self.heal_skills.insert(player_uid, skills_window);
    }

    pub fn get_dps_skills(&self, player_uid: i64) -> Option<&SkillsWindow> {
        self.dps_skills.get(&player_uid)
    }

    pub fn get_heal_skills(&self, player_uid: i64) -> Option<&SkillsWindow> {
        self.heal_skills.get(&player_uid)
    }

    pub fn subscribe(&mut self, player_uid: i64, skill_type: String) {
        self.active_subscriptions.insert((player_uid, skill_type));
    }

    pub fn unsubscribe(&mut self, player_uid: i64, skill_type: String) {
        self.active_subscriptions.remove(&(player_uid, skill_type));
    }

    pub fn is_subscribed(&self, player_uid: i64, skill_type: &str) -> bool {
        self.active_subscriptions.contains(&(player_uid, skill_type.to_string()))
    }

    pub fn get_active_subscriptions(&self) -> &HashSet<(i64, String)> {
        &self.active_subscriptions
    }

    pub fn clear_subscriptions(&mut self) {
        self.active_subscriptions.clear();
    }

    pub fn clear(&mut self) {
        self.dps_skills.clear();
        self.heal_skills.clear();
        self.active_subscriptions.clear();
    }
}

impl Default for SkillsStore {
    fn default() -> Self {
        Self::new()
    }
}

// Use an async RwLock so consumers can acquire read access without blocking.
pub type SkillsStoreMutex = RwLock<SkillsStore>;
