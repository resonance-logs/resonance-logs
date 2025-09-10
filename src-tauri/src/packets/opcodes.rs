#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PktParseError;

#[repr(u32)] // ensures the enum is stored as an u32
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pkt {
    // TODO: change all these names
    SyncNearEntities,            // NPCNearbyNotify SyncNearEntities
    DataNotifySyncContainerData, // Container DataNotifySyncContainerData - similar to DirtyData, but has detailed like level, curr hp, max hp
    SyncContainerDirtyData,      // DirtyDataNotify SyncContainerDirtyData - Name, AP, Class, SubClass
    SyncServerTime,              // ServerTimeNotify SyncServerTime
    SyncToMeDeltaInfo,           // PlayerSelfNotify SyncToMeDeltaInfo
    SyncNearDeltaInfo,           // PlayerNearbyNotify SyncNearDeltaInfo
}

impl TryFrom<u32> for Pkt {
    type Error = PktParseError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            // Can change these to use const mapping if we ever need the reverse mapping Pkt -> bytes
            0x00000006 => Ok(Pkt::SyncNearEntities),
            0x00000015 => Ok(Pkt::DataNotifySyncContainerData),
            0x00000016 => Ok(Pkt::SyncContainerDirtyData),
            0x0000002b => Ok(Pkt::SyncServerTime),
            0x0000002e => Ok(Pkt::SyncToMeDeltaInfo),
            0x0000002d => Ok(Pkt::SyncNearDeltaInfo),
            _ => Err(PktParseError),
        }
    }
}

/*
Pkt::DeathNotify
Pkt::IdentityGaugeChangeNotify
Pkt::IdentityStanceChangeNotify
Pkt::InitEnv
Pkt::InitPC
Pkt::InitItem
Pkt::MigrationExecute
Pkt::NewPC
Pkt::NewVehicle
Pkt::NewNpc
Pkt::NewNpcSummon
Pkt::NewProjectile
Pkt::NewTrap
Pkt::ParalyzationStateNotify
Pkt::RaidBegin
Pkt::RaidBossKillNotify
Pkt::RaidResult
Pkt::RemoveObject
Pkt::SkillCastNotify
Pkt::SkillCooldownNotify
Pkt::SkillStartNotify
Pkt::SkillStageNotify
Pkt::SkillDamageAbnormalMoveNotify
Pkt::SkillDamageNotify
Pkt::PartyInfo
Pkt::PartyLeaveResult
Pkt::PartyStatusEffectAddNotify
Pkt::PartyStatusEffectRemoveNotify
Pkt::PartyStatusEffectResultNotify
Pkt::StatusEffectAddNotify
Pkt::StatusEffectDurationNotify
Pkt::StatusEffectRemoveNotify
Pkt::TriggerBossBattleStatus
Pkt::TriggerStartNotify
Pkt::ZoneMemberLoadStatusNotify
Pkt::ZoneObjectUnpublishNotify
Pkt::StatusEffectSyncDataNotify
Pkt::TroopMemberUpdateMinNotify
Pkt::NewTransit
 */
