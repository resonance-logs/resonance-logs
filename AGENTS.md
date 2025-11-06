# Resonance-Logs appliation, Database Schema

### Core Tables

#### Encounters & Attempts

- **encounters**: Main encounter tracking table
  - `id`: Primary key (AUTOINCREMENT)
  - `started_at_ms`: Start timestamp in milliseconds
  - `ended_at_ms`: End timestamp in milliseconds (nullable)
  - `local_player_id`: ID of the local player (nullable)
  - `total_dmg`: Total damage dealt in encounter (default 0)
  - `total_heal`: Total healing done in encounter (default 0)
  - `scene_id`: Scene identifier (nullable)
  - `scene_name`: Scene/location name (nullable)

- **attempts**: Individual attempt tracking within encounters
  - `id`: Primary key (AUTOINCREMENT)
  - `encounter_id`: Foreign key to encounters
  - `attempt_index`: Attempt number within encounter
  - `started_at_ms`: Attempt start timestamp
  - `ended_at_ms`: Attempt end timestamp (nullable)
  - `reason`: Reason for attempt end ('wipe', 'hp_rollback', 'manual')
  - `boss_hp_start`: Boss HP at attempt start (nullable)
  - `boss_hp_end`: Boss HP at attempt end (nullable)
  - `total_deaths`: Total deaths in attempt (default 0)

- **encounter_bosses**: Boss information per encounter
  - `encounter_id`: Foreign key to encounters (composite PK)
  - `monster_name`: Boss/monster name (composite PK)
  - `hits`: Number of hits on boss (default 0)
  - `total_damage`: Total damage dealt to boss (default 0)
  - `max_hp`: Maximum boss health (nullable)
  - `is_defeated`: Boolean defeat status (default 0)

#### Player & Entity Data

- **entities**: Known entities (players only)
  - `entity_id`: Primary key (entity identifier)
  - `name`: Entity name (nullable)
  - `class_id`: Class identifier (nullable)
  - `class_spec`: Specialization identifier (nullable)
  - `ability_score`: Ability/gear score (nullable)
  - `level`: Entity level (nullable)
  - `first_seen_ms`: First seen timestamp (nullable)
  - `last_seen_ms`: Last seen timestamp (nullable)
  - `attributes`: JSON attributes (nullable)

- **actor_encounter_stats**: Per-encounter, per-actor aggregated stats
  - `encounter_id`: Foreign key to encounters (composite PK)
  - `actor_id`: Actor/entity identifier (composite PK)
  - `class_spec`: Class specialization (nullable)
  - `damage_dealt`: Total damage dealt (default 0)
  - `heal_dealt`: Total healing done (default 0)
  - `damage_taken`: Total damage taken (default 0)
  - `hits_dealt`: Number of hits dealt (default 0)
  - `hits_heal`: Number of heals done (default 0)
  - `hits_taken`: Number of hits taken (default 0)
  - `crit_hits_dealt`: Critical hits dealt (default 0)
  - `crit_hits_heal`: Critical heals done (default 0)
  - `crit_hits_taken`: Critical hits taken (default 0)
  - `lucky_hits_dealt`: Lucky hits dealt (default 0)
  - `lucky_hits_heal`: Lucky heals done (default 0)
  - `lucky_hits_taken`: Lucky hits taken (default 0)
  - `crit_total_dealt`: Total critical damage dealt (default 0)
  - `crit_total_heal`: Total critical healing done (default 0)
  - `crit_total_taken`: Total critical damage taken (default 0)
  - `lucky_total_dealt`: Total lucky damage dealt (default 0)
  - `lucky_total_heal`: Total lucky healing done (default 0)
  - `lucky_total_taken`: Total lucky damage taken (default 0)
  - `boss_damage_dealt`: Boss-only damage dealt (default 0)
  - `boss_hits_dealt`: Boss-only hits dealt (default 0)
  - `boss_crit_hits_dealt`: Boss-only critical hits dealt (default 0)
  - `boss_lucky_hits_dealt`: Boss-only lucky hits dealt (default 0)
  - `boss_crit_total_dealt`: Boss-only critical damage dealt (default 0)
  - `boss_lucky_total_dealt`: Boss-only lucky damage dealt (default 0)
  - `name`: Actor name snapshot (nullable)
  - `class_id`: Class ID snapshot (nullable)
  - `ability_score`: Ability score snapshot (nullable)
  - `level`: Level snapshot (nullable)
  - `is_player`: Boolean flag for player vs NPC (default 1)
  - `is_local_player`: Boolean flag for local player (default 0)
  - `attributes`: JSON attributes snapshot (nullable)
  - `revives`: Number of revives (default 0)

#### Combat Events

- **damage_events**: Detailed damage records
  - `id`: Primary key (AUTOINCREMENT)
  - `encounter_id`: Foreign key to encounters
  - `timestamp_ms`: Event timestamp in milliseconds
  - `attacker_id`: Attacker entity ID
  - `defender_id`: Defender entity ID (nullable)
  - `skill_id`: Skill/ability ID (nullable)
  - `value`: Damage amount
  - `is_crit`: Critical hit flag (default 0)
  - `is_lucky`: Lucky hit flag (default 0)
  - `hp_loss`: HP loss amount (default 0)
  - `shield_loss`: Shield loss amount (default 0)
  - `is_boss`: Boss damage flag (default 0)
  - `monster_name`: Monster name (nullable)
  - `defender_max_hp`: Defender max HP (nullable)
  - `attempt_index`: Attempt index (default 1)

- **heal_events**: Detailed healing records
  - `id`: Primary key (AUTOINCREMENT)
  - `encounter_id`: Foreign key to encounters
  - `timestamp_ms`: Event timestamp in milliseconds
  - `healer_id`: Healer entity ID
  - `target_id`: Target entity ID (nullable)
  - `skill_id`: Skill/ability ID (nullable)
  - `value`: Healing amount
  - `is_crit`: Critical heal flag (default 0)
  - `is_lucky`: Lucky heal flag (default 0)
  - `attempt_index`: Attempt index (default 1)

- **death_events**: Player death tracking
  - `id`: Primary key (AUTOINCREMENT)
  - `encounter_id`: Foreign key to encounters
  - `timestamp_ms`: Death timestamp in milliseconds
  - `actor_id`: Deceased actor ID
  - `killer_id`: Killer entity ID (nullable)
  - `skill_id`: Killing skill ID (nullable)
  - `is_local_player`: Local player death flag (default 0)
  - `attempt_index`: Attempt index (default 1)

#### Aggregated Data

- **damage_skill_stats**: Aggregated damage per skill/actor/defender
  - `encounter_id`: Foreign key to encounters (composite PK)
  - `attacker_id`: Attacker ID (composite PK)
  - `defender_id`: Defender ID (composite PK, nullable)
  - `skill_id`: Skill ID (composite PK)
  - `hits`: Number of hits (default 0)
  - `total_value`: Total damage value (default 0)
  - `crit_hits`: Critical hits (default 0)
  - `lucky_hits`: Lucky hits (default 0)
  - `crit_total`: Total critical damage (default 0)
  - `lucky_total`: Total lucky damage (default 0)
  - `hp_loss_total`: Total HP loss (default 0)
  - `shield_loss_total`: Total shield loss (default 0)
  - `hit_details`: JSON array of hit details
  - `monster_name`: Monster name (nullable)

- **heal_skill_stats**: Aggregated heals per skill/healer/target
  - `encounter_id`: Foreign key to encounters (composite PK)
  - `healer_id`: Healer ID (composite PK)
  - `target_id`: Target ID (composite PK, nullable)
  - `skill_id`: Skill ID (composite PK)
  - `hits`: Number of heals (default 0)
  - `total_value`: Total heal value (default 0)
  - `crit_hits`: Critical heals (default 0)
  - `lucky_hits`: Lucky heals (default 0)
  - `crit_total`: Total critical healing (default 0)
  - `lucky_total`: Total lucky healing (default 0)
  - `heal_details`: JSON array of heal details
  - `monster_name`: Monster name (nullable)

## Key Schema Relationships

```
encounters (1) → (N) attempts
encounters (1) → (N) encounter_bosses
encounters (1) → (N) damage_events
encounters (1) → (N) heal_events
encounters (1) → (N) death_events
encounters (1) → (N) actor_encounter_stats
encounters (1) → (N) damage_skill_stats
encounters (1) → (N) heal_skill_stats

entities (1) → (N) actor_encounter_stats (via actor_id)

attempts (N) → (1) encounters
encounter_bosses (N) → (1) encounters
damage_events (N) → (1) encounters
heal_events (N) → (1) encounters
death_events (N) → (1) encounters
actor_encounter_stats (N) → (1) encounters
damage_skill_stats (N) → (1) encounters
heal_skill_stats (N) → (1) encounters
```
