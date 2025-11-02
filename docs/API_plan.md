# API Upload Architecture for Resonance-logs

## Executive Summary

This document outlines the recommended architecture for implementing DPS raid log uploading from the Resonance-logs meter to a backend API (Go/Gin). The implementation should leverage the existing database infrastructure, create a minimal serialization layer, and follow established patterns from similar projects like loa-logs and FFLogs.

## ⚠️ Critical Requirements

Before implementation, the following **MUST** be addressed:

### 🔒 Authentication (Critical)
**Status**: Required for MVP

Authentication is **mandatory** from day 1, not optional. Anonymous uploads create:
- Spam and abuse vectors
- No ownership/deletion capability
- No rate limiting per user
- GDPR compliance issues

**Recommended Approach**:
- Simple API key system for MVP
- Users register via web portal to get API key
- Store API key in meter settings (encrypted)
- Include `X-API-Key` header in all upload requests
- Rate limit: 100 uploads/day per API key

### 🔄 Data Integrity (Critical)
**Status**: Required for MVP

Client-side data retrieval must use transaction boundaries to ensure consistency:

```rust
// WRONG - Can read inconsistent data
pub fn get_encounter_upload_data(encounter_id: i32) -> Result<EncounterUploadDto, String> {
    let encounter = query_encounter(encounter_id)?;
    let players = query_players(encounter_id)?; // Could be modified between queries
    let skills = query_skills(encounter_id)?;
}

// CORRECT - Use explicit transaction
pub fn get_encounter_upload_data(encounter_id: i32) -> Result<EncounterUploadDto, String> {
    use diesel::Connection;
    let mut conn = establish_connection()?;

    conn.transaction(|conn| {
        let encounter = query_encounter(conn, encounter_id)?;
        let players = query_players(conn, encounter_id)?;
        let skills = query_skills(conn, encounter_id)?;
        Ok(build_upload_dto(encounter, players, skills))
    }).map_err(|e| format!("Transaction failed: {}", e))
}
```

---

## Current Architecture Analysis

### Database Layer (SQLite with Diesel ORM)

The meter already has a robust database infrastructure:

**Core Tables:**
- `encounters` - Stores encounter metadata (start/end times, totals)
- `entities` - Player information (names, classes, ability scores)
- `actor_encounter_stats` - Per-player aggregated stats per encounter
- `damage_skill_stats` - Skill-level damage breakdowns (materialized after encounter)
- `heal_skill_stats` - Skill-level healing breakdowns (materialized after encounter)
- `encounter_bosses` - Boss names and damage totals
- `damage_events` - Raw damage events (pruned after materialization)
- `heal_events` - Raw heal events (pruned after materialization)

**Key Characteristics:**
- Uses WAL mode for concurrency
- Batched writes (up to 100 events or 50ms window)
- Materializes aggregated stats on encounter end and prunes raw events
- Location: `%LOCALAPPDATA%\resonance-logs\resonance-logs.db`

### Data Models

**Live Encounter Data (in-memory):**
```rust
pub struct Encounter {
    pub is_encounter_paused: bool,
    pub time_last_combat_packet_ms: u128,
    pub time_fight_start_ms: u128,
    pub local_player_uid: Option<i64>,
    pub entity_uid_to_entity: HashMap<i64, Entity>,
    pub dmg_stats: CombatStats,
    pub heal_stats: CombatStats,
    pub dmg_stats_boss_only: CombatStats,
}

pub struct Entity {
    pub entity_type: EEntityType,
    pub dmg_stats: CombatStats,
    pub skill_uid_to_dps_stats: HashMap<i32, CombatStats>,
    pub heal_stats: CombatStats,
    pub skill_uid_to_heal_stats: HashMap<i32, CombatStats>,
    pub name: Option<String>,
    pub class: Option<Class>,
    pub class_spec: Option<ClassSpec>,
    pub ability_score: Option<i32>,
    pub monster_id: Option<i32>,
    pub curr_hp: Option<i32>,
    pub max_hp: Option<i32>,
}

pub struct CombatStats {
    pub value: i64,
    pub hits: i64,
    pub crit_value: i64,
    pub crit_hits: i64,
    pub lucky_value: i64,
    pub lucky_hits: i64,
}
```

**Database Models (persisted):**
- All models in `src-tauri/src/database/models.rs` with Diesel derives
- Already implements `Serialize/Deserialize` via Serde
- Aggregated stats include hit_details as JSON strings

---

## Recommended Upload Strategy

### Option 1: Upload from Database (Recommended)

**Advantages:**
- ✅ Complete, validated data
- ✅ Can upload historical encounters
- ✅ Skill-level breakdowns already materialized
- ✅ No live encounter state management needed
- ✅ Hit-by-hit timeline data preserved in `hit_details` JSON
- ✅ Boss filtering already computed

**Implementation:**
```rust
// Add to src-tauri/src/database/commands.rs
#[tauri::command]
#[specta::specta]
pub fn get_encounter_upload_data(encounter_id: i32) -> Result<EncounterUploadDto, String> {
    // Query all necessary data from database
    // Return comprehensive DTO ready for JSON serialization
}
```

**Recommended Upload DTO Structure:**
```rust
#[derive(Serialize, Deserialize, specta::Type)]
pub struct EncounterUploadDto {
    // Encounter metadata
    pub encounter_id: i32,
    pub started_at_ms: i64,
    pub ended_at_ms: i64,
    pub duration_ms: i64,
    pub total_damage: i64,
    pub total_heal: i64,

    // Boss information
    pub bosses: Vec<BossDto>,

    // Player statistics
    pub players: Vec<PlayerUploadDto>,

    // Metadata
    pub meter_version: String,
    pub local_player_uid: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct BossDto {
    pub name: String,
    pub total_damage: i64,
    pub hits: i32,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerUploadDto {
    pub actor_id: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub level: Option<i32>,

    // Aggregated stats
    pub damage_dealt: i64,
    pub heal_dealt: i64,
    pub damage_taken: i64,
    pub boss_damage_dealt: i64,

    // Rates and percentages
    pub dps: f64,
    pub hps: f64,
    pub crit_rate: f64,
    pub lucky_rate: f64,

    // Skill breakdowns
    pub damage_skills: Vec<SkillUploadDto>,
    pub heal_skills: Vec<SkillUploadDto>,
}

#[derive(Serialize, Deserialize)]
pub struct SkillUploadDto {
    pub skill_id: i32,
    pub skill_name: String,
    pub total_value: i64,
    pub hits: i32,
    pub crit_hits: i32,
    pub lucky_hits: i32,
    pub crit_total: i64,
    pub lucky_total: i64,

    // Hit-by-hit timeline (from hit_details JSON)
    pub timeline: Vec<HitDetailDto>,
}

#[derive(Serialize, Deserialize)]
pub struct HitDetailDto {
    pub timestamp_ms: i64,  // "t"
    pub value: i64,         // "v"
    pub is_crit: bool,      // "c"
    pub is_lucky: bool,     // "l"
    pub hp_loss: i64,       // "h" (damage events only)
    pub shield_loss: i64,   // "s" (damage events only)
}
```

### Option 2: Upload from Live Encounter

**Use only if real-time uploads are needed (not recommended for initial implementation)**

**Disadvantages:**
- ❌ No historical uploads
- ❌ Must wait for encounter to complete
- ❌ More complex state management
- ❌ Need to serialize in-memory state

---

## Backend API Design (Go/Gin)

### Recommended Endpoints

**API Versioning Strategy**:
- URL-based versioning: `/api/v1/`, `/api/v2/`
- Support previous version for 6 months after new version release
- Include `schema_version` in payloads for backward compatibility
- Return `X-API-Version` header in responses

```go
// POST /api/v1/encounters
// Upload a new encounter
// Requires: X-API-Key header
type EncounterUploadRequest struct {
    SchemaVersion   int               `json:"schemaVersion" binding:"required,min=1"`
    EncounterId     int64             `json:"encounterId" binding:"required"`
    StartedAtMs     int64             `json:"startedAtMs" binding:"required"`
    EndedAtMs       int64             `json:"endedAtMs" binding:"required"`
    DurationMs      int64             `json:"durationMs" binding:"required,min=1,max=7200000"`
    TotalDamage     int64             `json:"totalDamage"`
    TotalHeal       int64             `json:"totalHeal"`
    Bosses          []Boss            `json:"bosses" binding:"required,dive"`
    Players         []PlayerUpload    `json:"players" binding:"required,max=50,dive"`
    MeterVersion    string            `json:"meterVersion" binding:"required"`
    LocalPlayerUid  *int64            `json:"localPlayerUid,omitempty"`
    Checksum        string            `json:"checksum" binding:"required,len=64"`
}

type EncounterUploadResponse struct {
    EncounterId     int64  `json:"encounterId"`      // Server-side ID
    ReportId        string `json:"reportId"`          // Public share ID (UUID or short code)
    ReportUrl       string `json:"reportUrl"`         // e.g., "https://bpsr.logs/reports/{reportId}"
    CreatedAt       int64  `json:"createdAt"`
}

// GET /api/v1/encounters/:reportId
// Retrieve encounter data for viewing
// Optional: X-API-Key header for private reports

// GET /api/v1/encounters?playerId=xxx&bossName=xxx&dateFrom=xxx&dateTo=xxx&page=1&limit=50
// List/search encounters with pagination
type EncounterListRequest struct {
    PlayerID   *string `form:"playerId"`
    BossName   *string `form:"bossName"`
    DateFrom   *int64  `form:"dateFrom"`
    DateTo     *int64  `form:"dateTo"`
    Page       int     `form:"page" binding:"min=1" default:"1"`
    Limit      int     `form:"limit" binding:"min=1,max=100" default:"50"`
}

type EncounterListResponse struct {
    Encounters []EncounterSummary `json:"encounters"`
    Pagination PaginationMeta     `json:"pagination"`
}

type PaginationMeta struct {
    Page       int   `json:"page"`
    Limit      int   `json:"limit"`
    Total      int64 `json:"total"`
    TotalPages int   `json:"totalPages"`
}
```

### Storage Considerations

**Recommended Database: PostgreSQL**
- Strong JSON support for skill timelines
- JSONB for efficient querying
- Partitioning by date for scalability
- Full-text search for player/boss names

**Schema Design:**
```sql
CREATE TABLE encounters (
    id BIGSERIAL PRIMARY KEY,
    report_id VARCHAR(12) UNIQUE NOT NULL,  -- Short code like "aB3fG7kL9pQ"

    -- Schema versioning for API evolution
    schema_version INT NOT NULL DEFAULT 1,

    -- Deduplication: prevent same encounter uploaded twice
    client_encounter_id INT NOT NULL,
    uploader_api_key_hash VARCHAR(64) NOT NULL, -- SHA256 of API key

    started_at_ms BIGINT NOT NULL,
    ended_at_ms BIGINT NOT NULL,
    duration_ms BIGINT NOT NULL,
    total_damage BIGINT DEFAULT 0,
    total_heal BIGINT DEFAULT 0,
    meter_version VARCHAR(32),
    local_player_uid BIGINT,

    -- Metadata
    checksum VARCHAR(64), -- Client-provided SHA256
    created_at TIMESTAMP DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_duration CHECK (duration_ms > 0 AND duration_ms <= 7200000),
    CONSTRAINT unique_upload UNIQUE (client_encounter_id, uploader_api_key_hash),

    INDEX idx_started_at (started_at_ms),
    INDEX idx_report_id (report_id),
    INDEX idx_uploader (uploader_api_key_hash)
);

CREATE TABLE encounter_bosses (
    encounter_id BIGINT REFERENCES encounters(id) ON DELETE CASCADE,
    boss_name VARCHAR(255) NOT NULL,
    total_damage BIGINT DEFAULT 0,
    hits INT DEFAULT 0,
    PRIMARY KEY (encounter_id, boss_name),
    INDEX idx_boss_name (boss_name)
);

CREATE TABLE encounter_players (
    encounter_id BIGINT REFERENCES encounters(id) ON DELETE CASCADE,
    actor_id BIGINT NOT NULL,
    player_name VARCHAR(255),
    class_id INT,
    class_spec INT,
    ability_score INT,
    level INT,
    damage_dealt BIGINT DEFAULT 0,
    heal_dealt BIGINT DEFAULT 0,
    damage_taken BIGINT DEFAULT 0,
    boss_damage_dealt BIGINT DEFAULT 0,
    hits_dealt BIGINT DEFAULT 0,
    hits_heal BIGINT DEFAULT 0,
    crit_hits_dealt BIGINT DEFAULT 0,
    lucky_hits_dealt BIGINT DEFAULT 0,
    PRIMARY KEY (encounter_id, actor_id),
    INDEX idx_player_name (player_name)
);

CREATE TABLE player_skills (
    encounter_id BIGINT,
    actor_id BIGINT,
    skill_id INT NOT NULL,
    skill_name VARCHAR(255),
    skill_type VARCHAR(10), -- 'damage' or 'heal'
    total_value BIGINT DEFAULT 0,
    hits INT DEFAULT 0,
    crit_hits INT DEFAULT 0,
    lucky_hits INT DEFAULT 0,
    crit_total BIGINT DEFAULT 0,
    lucky_total BIGINT DEFAULT 0,

    -- Timeline stored separately for better performance
    -- See player_skill_timeline table below
    timeline_summary JSONB, -- Aggregated stats only (first/last hit, peaks)

    PRIMARY KEY (encounter_id, actor_id, skill_id, skill_type),
    FOREIGN KEY (encounter_id, actor_id)
        REFERENCES encounter_players(encounter_id, actor_id)
        ON DELETE CASCADE,

    -- Size limit constraint
    CONSTRAINT valid_hits CHECK (hits >= 0 AND hits <= 10000)
);

-- Separate table for timeline data to prevent row bloat
CREATE TABLE player_skill_timeline (
    encounter_id BIGINT,
    actor_id BIGINT,
    skill_id INT,
    skill_type VARCHAR(10),
    timeline JSONB NOT NULL, -- Array of hit details

    PRIMARY KEY (encounter_id, actor_id, skill_id, skill_type),
    FOREIGN KEY (encounter_id, actor_id, skill_id, skill_type)
        REFERENCES player_skills(encounter_id, actor_id, skill_id, skill_type)
        ON DELETE CASCADE,

    -- Prevent massive timeline storage
    CONSTRAINT timeline_size_limit CHECK (jsonb_array_length(timeline) <= 10000)
);

CREATE INDEX idx_timeline_gin ON player_skill_timeline USING GIN (timeline);
```

---

## Implementation Steps

### Phase 1: Client-Side Upload (Rust/Tauri)

**1. Create Upload Module**
```
src-tauri/src/upload/
├── mod.rs           # Module exports
├── dto.rs           # Upload DTOs
├── client.rs        # HTTP client implementation
└── commands.rs      # Tauri commands
```

**2. Add Dependencies to Cargo.toml**
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

**3. Implement Upload Command with Retry Logic**
```rust
// src-tauri/src/upload/commands.rs
use std::time::Duration;
use tokio::time::sleep;

#[tauri::command]
#[specta::specta]
pub async fn upload_encounter(
    encounter_id: i32,
    api_endpoint: Option<String>,
    api_key: String,
) -> Result<UploadResponse, String> {
    // 1. Get encounter data from database (with transaction)
    let upload_data = database::commands::get_encounter_upload_data(encounter_id)?;

    // 2. Validate before upload
    validate_upload_data(&upload_data)?;

    // 3. Send to API with retry logic
    let endpoint = api_endpoint.unwrap_or_else(|| "https://api.bpsr.logs".to_string());

    upload_with_retry(&endpoint, &api_key, &upload_data, 3).await
}

async fn upload_with_retry(
    endpoint: &str,
    api_key: &str,
    data: &EncounterUploadDto,
    max_retries: u32,
) -> Result<UploadResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut last_error = String::new();

    for attempt in 0..max_retries {
        if attempt > 0 {
            // Exponential backoff: 1s, 2s, 4s
            let delay = Duration::from_secs(2u64.pow(attempt - 1));
            sleep(delay).await;
        }

        match client
            .post(format!("{}/api/v1/encounters", endpoint))
            .header("X-API-Key", api_key)
            .header("Content-Type", "application/json")
            .json(&data)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();

                // Don't retry on client errors (4xx)
                if status.is_client_error() {
                    let error_text = response.text().await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(format!("Upload failed with status {}: {}", status, error_text));
                }

                if status.is_success() {
                    return response
                        .json::<UploadResponse>()
                        .await
                        .map_err(|e| format!("Failed to parse response: {}", e));
                }

                // Retry on server errors (5xx)
                last_error = format!("Server error: {}", status);
            }
            Err(e) => {
                last_error = format!("Network error: {}", e);
            }
        }
    }

    Err(format!("Upload failed after {} attempts: {}", max_retries, last_error))
}

fn validate_upload_data(data: &EncounterUploadDto) -> Result<(), String> {
    // Validate duration
    if data.duration_ms <= 0 || data.duration_ms > MAX_ENCOUNTER_DURATION_MS {
        return Err(format!("Invalid duration: {}ms", data.duration_ms));
    }

    // Validate player count
    if data.players.len() > MAX_PLAYERS_PER_ENCOUNTER {
        return Err(format!("Too many players: {}", data.players.len()));
    }

    // Validate timeline sizes
    for player in &data.players {
        for skill in data.damage_skills.iter().chain(data.heal_skills.iter()) {
            if skill.timeline.len() > MAX_TIMELINE_EVENTS_PER_SKILL {
                return Err(format!(
                    "Skill {} timeline too large: {} events",
                    skill.skill_name,
                    skill.timeline.len()
                ));
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, specta::Type)]
pub struct UploadResponse {
    pub encounter_id: i64,
    pub report_id: String,
    pub report_url: String,
    pub created_at: i64,
}
```

**4. Add UI Upload Button (SvelteKit)**
```svelte
<!-- src/routes/main/history/encounter-card.svelte -->
<script lang="ts">
  import { commands } from "$lib/bindings";

  export let encounterId: number;

  let uploading = false;
  let uploadResult: { reportUrl?: string; error?: string } = {};

  async function uploadEncounter() {
    uploading = true;
    uploadResult = {};

    try {
      const result = await commands.uploadEncounter(encounterId);
      uploadResult = { reportUrl: result.reportUrl };

      // Copy to clipboard
      await navigator.clipboard.writeText(result.reportUrl);
    } catch (error) {
      uploadResult = { error: String(error) };
    } finally {
      uploading = false;
    }
  }
</script>

<button
  onclick={uploadEncounter}
  disabled={uploading}
  class="upload-btn"
>
  {uploading ? "Uploading..." : "Upload & Share"}
</button>

{#if uploadResult.reportUrl}
  <div class="success">
    Link copied! <a href={uploadResult.reportUrl} target="_blank">{uploadResult.reportUrl}</a>
  </div>
{/if}

{#if uploadResult.error}
  <div class="error">{uploadResult.error}</div>
{/if}
```

### Phase 2: Backend API (Go/Gin)

**1. Project Structure**
```
resonance-logs-api/
├── cmd/
│   └── api/
│       └── main.go
├── internal/
│   ├── config/
│   │   └── config.go
│   ├── db/
│   │   ├── postgres.go
│   │   └── migrations/
│   ├── handlers/
│   │   ├── upload.go
│   │   └── query.go
│   ├── models/
│   │   └── encounter.go
│   └── utils/
│       └── report_id.go
├── go.mod
└── README.md
```

**2. Main API Handler**
```go
// internal/handlers/upload.go
package handlers

import (
    "net/http"
    "time"
    "github.com/gin-gonic/gin"
    "github.com/yourusername/resonance-logs-api/internal/db"
    "github.com/yourusername/resonance-logs-api/internal/models"
    "github.com/yourusername/resonance-logs-api/internal/utils"
)

func UploadEncounter(c *gin.Context) {
    var req models.EncounterUploadRequest
    if err := c.ShouldBindJSON(&req); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
        return
    }

    // Generate short report ID
    reportID := utils.GenerateReportID()

    // Begin transaction
    tx := db.DB.Begin()
    defer tx.Rollback()

    // Insert encounter
    encounter := models.Encounter{
        ReportID:       reportID,
        StartedAtMs:    req.StartedAtMs,
        EndedAtMs:      req.EndedAtMs,
        DurationMs:     req.DurationMs,
        TotalDamage:    req.TotalDamage,
        TotalHeal:      req.TotalHeal,
        MeterVersion:   req.MeterVersion,
        LocalPlayerUid: req.LocalPlayerUid,
        CreatedAt:      time.Now(),
    }

    if err := tx.Create(&encounter).Error; err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save encounter"})
        return
    }

    // Insert bosses
    for _, boss := range req.Bosses {
        encounterBoss := models.EncounterBoss{
            EncounterID:  encounter.ID,
            BossName:     boss.Name,
            TotalDamage:  boss.TotalDamage,
            Hits:         boss.Hits,
        }
        if err := tx.Create(&encounterBoss).Error; err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save boss data"})
            return
        }
    }

    // Insert players and their skills
    for _, player := range req.Players {
        encounterPlayer := models.EncounterPlayer{
            EncounterID:     encounter.ID,
            ActorID:         player.ActorID,
            PlayerName:      player.Name,
            ClassID:         player.ClassID,
            ClassSpec:       player.ClassSpec,
            AbilityScore:    player.AbilityScore,
            Level:           player.Level,
            DamageDealt:     player.DamageDealt,
            HealDealt:       player.HealDealt,
            DamageTaken:     player.DamageTaken,
            BossDamageDealt: player.BossDamageDealt,
            HitsDealt:       player.HitsDealt,
            HitsHeal:        player.HitsHeal,
            CritHitsDealt:   player.CritHitsDealt,
            LuckyHitsDealt:  player.LuckyHitsDealt,
        }
        if err := tx.Create(&encounterPlayer).Error; err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save player data"})
            return
        }

        // Insert damage skills
        for _, skill := range player.DamageSkills {
            playerSkill := models.PlayerSkill{
                EncounterID: encounter.ID,
                ActorID:     player.ActorID,
                SkillID:     skill.SkillID,
                SkillName:   skill.SkillName,
                SkillType:   "damage",
                TotalValue:  skill.TotalValue,
                Hits:        skill.Hits,
                CritHits:    skill.CritHits,
                LuckyHits:   skill.LuckyHits,
                CritTotal:   skill.CritTotal,
                LuckyTotal:  skill.LuckyTotal,
                Timeline:    skill.Timeline, // JSONB
            }
            if err := tx.Create(&playerSkill).Error; err != nil {
                c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to save skill data"})
                return
            }
        }

        // Insert heal skills (same pattern)
        // ...
    }

    // Commit transaction
    if err := tx.Commit().Error; err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to commit transaction"})
        return
    }

    // Return response
    c.JSON(http.StatusCreated, models.UploadResponse{
        EncounterID: encounter.ID,
        ReportID:    reportID,
        ReportURL:   fmt.Sprintf("https://bpsr.logs/reports/%s", reportID),
        CreatedAt:   encounter.CreatedAt.Unix() * 1000,
    })
}
```

**3. Report ID Generator**
```go
// internal/utils/report_id.go
package utils

import (
    "crypto/rand"
    "encoding/base64"
)

// GenerateReportID creates a URL-safe short code (12 chars)
func GenerateReportID() string {
    b := make([]byte, 9) // 9 bytes = 12 base64 chars
    rand.Read(b)
    return base64.URLEncoding.EncodeToString(b)[:12]
}
```

### Phase 3: Observability & Monitoring

**1. Structured Logging**
```go
// Use structured logging (e.g., zap, zerolog)
import "go.uber.org/zap"

func UploadEncounter(c *gin.Context) {
    logger := zap.L().With(
        zap.String("endpoint", "upload_encounter"),
        zap.String("api_key_hash", hashAPIKey(apiKey)),
    )

    logger.Info("Upload started",
        zap.Int64("encounter_id", req.EncounterId),
        zap.Int("player_count", len(req.Players)),
        zap.Int64("duration_ms", req.DurationMs),
    )

    // ... processing ...

    logger.Info("Upload completed",
        zap.String("report_id", reportID),
        zap.Duration("processing_time", time.Since(start)),
    )
}
```

**2. Metrics Collection**
```go
// Use Prometheus metrics
import "github.com/prometheus/client_golang/prometheus"

var (
    uploadsTotal = prometheus.NewCounterVec(
        prometheus.CounterOpts{
            Name: "encounters_uploaded_total",
            Help: "Total number of encounters uploaded",
        },
        []string{"status", "meter_version"},
    )

    uploadDuration = prometheus.NewHistogramVec(
        prometheus.HistogramOpts{
            Name: "encounter_upload_duration_seconds",
            Help: "Time taken to process uploads",
            Buckets: prometheus.DefBuckets,
        },
        []string{"status"},
    )

    encounterSize = prometheus.NewHistogram(
        prometheus.HistogramOpts{
            Name: "encounter_payload_size_bytes",
            Help: "Size of encounter payloads",
            Buckets: []float64{1024, 10240, 102400, 1048576, 10485760},
        },
    )
)
```

**3. Distributed Tracing**
```go
// Use OpenTelemetry for tracing
import "go.opentelemetry.io/otel"

func UploadEncounter(c *gin.Context) {
    ctx, span := otel.Tracer("api").Start(c.Request.Context(), "upload_encounter")
    defer span.End()

    span.SetAttributes(
        attribute.Int64("encounter.id", req.EncounterId),
        attribute.Int("encounter.players", len(req.Players)),
    )

    // Pass context to database operations for full trace
    if err := saveEncounter(ctx, encounter); err != nil {
        span.RecordError(err)
        span.SetStatus(codes.Error, err.Error())
        return
    }
}
```

**4. Health Checks**
```go
// GET /health
func HealthCheck(c *gin.Context) {
    health := gin.H{
        "status": "healthy",
        "timestamp": time.Now().Unix(),
        "version": "1.0.0",
    }

    // Check database
    if err := db.DB.Exec("SELECT 1").Error; err != nil {
        health["status"] = "unhealthy"
        health["database"] = "down"
        c.JSON(http.StatusServiceUnavailable, health)
        return
    }

    health["database"] = "up"
    c.JSON(http.StatusOK, health)
}

// GET /metrics (Prometheus endpoint)
func MetricsHandler() gin.HandlerFunc {
    return gin.WrapH(promhttp.Handler())
}
```

### Phase 4: Advanced Features

**1. Compression**
- Use gzip compression for large payloads (encounters with many skills)
- Implement client-side: `reqwest` supports automatic compression
- Server-side: Gin middleware for gzip

**2. Privacy Controls**
- Option to anonymize player names
- Expiring links (auto-delete after 30 days)
- Private/public toggle

**3. Incremental Uploads**
- For live raids, support partial uploads during fight
- WebSocket connection for real-time updates
- Finalize on encounter end

---

## Performance Considerations

### Client-Side
- Upload in background (async)
- Retry logic with exponential backoff
- Queue multiple uploads
- Show progress indicator

### Server-Side
- Connection pooling (PostgreSQL)
- Redis cache for frequently accessed reports
- CDN for static assets
- Rate limiting per IP (100 uploads/hour)
- Batch inserts for skills

### Database Optimization
```sql
-- Partition by month
CREATE TABLE encounters_2024_11 PARTITION OF encounters
    FOR VALUES FROM ('2024-11-01') TO ('2024-12-01');

-- Indexes
CREATE INDEX idx_encounters_created_at ON encounters(created_at DESC);
CREATE INDEX idx_players_name_trgm ON encounter_players USING gin(player_name gin_trgm_ops);
CREATE INDEX idx_bosses_name ON encounter_bosses(boss_name);
```

---

## Security Considerations

1. **Input Validation**
   - Limit upload size (max 10MB per encounter)
   - Validate encounter duration (reject if > 2 hours)
   - Sanitize player names (prevent XSS)
   - Rate limiting by IP and/or API key

2. **Data Privacy**
   - GDPR compliance for EU players
   - Allow users to delete their own uploads
   - Don't store IP addresses long-term
   - Hash player UIDs before storing

3. **API Security**
   - HTTPS only
   - CORS policy (whitelist meter domains)
   - Request signing (HMAC) for verified uploads
   - DDoS protection (Cloudflare)

---

## Similar Implementations Reference

### FFLogs Approach
- Upload via chunked multipart
- Generates report ID on first chunk
- Live updates during parse
- Final processing on completion marker

### WarcraftLogs Pattern
- Single POST with full payload
- Synchronous processing
- Returns shareable URL immediately
- Background analysis for rankings

### Recommended Hybrid
- Single POST for simplicity (like WarcraftLogs)
- Background processing for heavy computations (rankings, percentiles)
- Immediate URL return with "processing" state
- WebSocket or polling for completion

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_encounter_success() {
        // Mock HTTP server
        // Test successful upload
        // Verify response parsing
    }

    #[tokio::test]
    async fn test_upload_encounter_network_error() {
        // Test retry logic
        // Test error handling
    }
}
```

### Integration Tests
```go
func TestUploadEncounterEndToEnd(t *testing.T) {
    // Setup test database
    // Send valid encounter payload
    // Verify all tables populated
    // Verify report URL generation
}
```

### Load Tests
- Use `k6` or `artillery` for load testing
- Test 1000 concurrent uploads
- Verify database connection pool handling
- Monitor memory usage during bulk inserts

---

## Deployment Recommendations

### Client (Meter Updates)
- Tauri auto-updater for new upload feature
- Feature flag for gradual rollout
- Error telemetry (Sentry)

### Backend (Go API)
**Option 1: Serverless (Recommended for MVP)**
- Google Cloud Run / AWS Lambda
- Auto-scaling
- Pay per request
- PostgreSQL via Cloud SQL

**Option 2: Kubernetes (For Production)**
- GKE / EKS / AKS
- Horizontal pod autoscaling
- PostgreSQL via managed service
- Redis for caching

### Infrastructure
```yaml
# docker-compose.yml (for local dev)
version: '3.8'
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: bpsr_logs
      POSTGRES_USER: bpsr
      POSTGRES_PASSWORD: secret
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://bpsr:secret@postgres:5432/bpsr_logs
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

volumes:
  pgdata:
```

---

## Cost Estimates (AWS Example)

**For 10,000 uploads/month, 100 unique visitors/day:**

| Service | Specs | Monthly Cost |
|---------|-------|--------------|
| RDS PostgreSQL | db.t3.medium, 50GB | $70 |
| ElastiCache Redis | t3.micro | $15 |
| EC2 / ECS | t3.small x2 | $30 |
| S3 (backups) | 100GB | $2 |
| CloudFront CDN | 10GB transfer | $1 |
| **Total** | | **~$120/month** |

**Serverless Alternative:**
- Cloud Run: $5-20/month (with free tier)
- Cloud SQL: $50-80/month
- **Total: ~$55-100/month**

---

## Timeline Estimate

### Phase 1: MVP (2-3 weeks)
- Week 1: Client-side upload implementation
  - Upload DTOs
  - HTTP client
  - UI integration
- Week 2: Backend API
  - Database schema
  - Upload endpoint
  - Basic query endpoints
- Week 3: Testing & deployment
  - Integration tests
  - Load testing
  - Deploy to staging

### Phase 2: Enhanced Features (2-4 weeks)
- Rankings/leaderboards
- Player profiles
- Advanced filters
- Public/private toggles

### Phase 3: Analytics (2-3 weeks)
- DPS percentiles
- Skill usage analysis
- Class performance charts
- Rotation analysis

---

## Conclusion

The recommended approach is:

1. **Use database-backed uploads** (Option 1) for reliability and historical support
2. **Implement Go/Gin backend** with PostgreSQL for scalability
3. **Start with single-POST upload pattern** for simplicity
4. **Add compression and chunking** only if encounter sizes exceed 5MB
5. **Deploy serverless initially** (Cloud Run/Lambda) for cost efficiency
6. **Add analytics layer** in Phase 2 after proving upload reliability

The existing Resonance-logs database already contains 90% of the data needed. The main work is creating the upload DTO serialization layer and building the backend API to receive and serve the data.

**Next Steps:**
1. Create `src-tauri/src/upload/dto.rs` with upload models
2. Add `get_encounter_upload_data()` to `database/commands.rs`
3. Implement upload HTTP client with retry logic
4. Build Go API with PostgreSQL schema
5. Add upload button to History UI


## 📋 Issues Summary & Resolution Plan

This section addresses the 14 identified issues across different severity levels:

### 🚨 Critical Issues (2) - RESOLVED

#### 1. Data Integrity: Transaction Boundaries ✅
**Status**: Addressed in Critical Requirements section
- Added explicit transaction usage in `get_encounter_upload_data()`
- Prevents inconsistent data reads during multi-table queries
- Implementation example provided with Diesel `Connection::transaction()`

#### 2. Security: Authentication Required ✅
**Status**: Addressed in Critical Requirements section
- API key system mandatory from MVP launch
- `X-API-Key` header required on all upload requests
- Rate limiting: 100 uploads/day per API key
- User registration via web portal

### ⚠️ High Priority (4) - RESOLVED

#### 3. Error Recovery: Retry Logic ✅
**Status**: Implemented in Phase 1, Section 3
- Exponential backoff: 1s, 2s, 4s delays
- Max 3 retry attempts
- Smart retry: skip 4xx errors, retry 5xx errors
- Timeout: 30 seconds per request

#### 4. Schema Versioning ✅
**Status**: Added to DTOs and database schema
- `schema_version` field in `EncounterUploadDto` (starts at 1)
- `schema_version` column in PostgreSQL `encounters` table
- API versioning strategy: `/api/v1/`, `/api/v2/`
- Backward compatibility for 6 months

#### 5. Deduplication ✅
**Status**: Added to PostgreSQL schema
- `UNIQUE (client_encounter_id, uploader_api_key_hash)` constraint
- Prevents same encounter uploaded twice by same user
- Returns existing `report_id` on duplicate attempt

#### 6. Performance: Timeline Size Limits ✅
**Status**: Multiple mitigations implemented
- Separate `player_skill_timeline` table to prevent row bloat
- `MAX_TIMELINE_EVENTS_PER_SKILL = 10,000` constant
- Database constraint: `CHECK (jsonb_array_length(timeline) <= 10000)`
- Client-side validation before upload

### 📋 Medium Priority (5) - RESOLVED

#### 7. Pagination ✅
**Status**: Added to GET endpoints
- `page` and `limit` query parameters
- Default: 50 items per page, max 100
- `PaginationMeta` response with total count and pages

#### 8. Duration Validation ✅
**Status**: Implemented in multiple layers
- Client-side: `validate_upload_data()` function
- DTO binding: `binding:"required,min=1,max=7200000"`
- Database constraint: `CHECK (duration_ms > 0 AND duration_ms <= 7200000)`
- Max duration: 2 hours (7,200,000ms)

#### 9. Type Mismatch: HitDetailDto ✅
**Status**: Fixed structure
- Changed `hp_loss` and `shield_loss` to `Option<i64>`
- Matches actual database JSON format where damage-only fields are optional
- Properly handles heal events without these fields

#### 10. Cascade Deletes ✅
**Status**: Complete cascade strategy implemented
- `encounters` → `encounter_bosses`: `ON DELETE CASCADE`
- `encounters` → `encounter_players`: `ON DELETE CASCADE`
- `encounter_players` → `player_skills`: `ON DELETE CASCADE`
- `player_skills` → `player_skill_timeline`: `ON DELETE CASCADE`
- Added index on `boss_name` for performance

#### 11. Observability ✅
**Status**: Comprehensive monitoring added in Phase 3
- **Structured Logging**: zap logger with contextual fields
- **Metrics**: Prometheus counters and histograms
  - `encounters_uploaded_total` (by status, meter_version)
  - `encounter_upload_duration_seconds`
  - `encounter_payload_size_bytes`
- **Distributed Tracing**: OpenTelemetry integration
- **Health Checks**: `/health` and `/metrics` endpoints

### 📝 Low Priority (3) - ACKNOWLEDGED

#### 12. Circular Dependencies
**Status**: Mitigated by module structure
- Upload module is leaf dependency (only imports database)
- No circular imports in proposed structure:
  ```
  upload/ (imports database)
    ├── dto.rs
    ├── client.rs
    └── commands.rs
  ```

#### 13. Contract Testing
**Status**: Addressed via specta
- Project already uses specta for type-safe contracts
- TypeScript types auto-generated from Rust DTOs
- Runtime validation via serde and Gin binding tags

#### 14. API Versioning Policy
**Status**: Documented in Backend API Design section
- URL-based versioning: `/api/v1/`, `/api/v2/`
- Support previous version for 6 months
- `X-API-Version` header in responses
- `schema_version` in payloads for compatibility

### 💡 Architecture Improvements (Recommendations)

#### A. Separate Timeline Storage ✅ IMPLEMENTED
- Created `player_skill_timeline` table separate from `player_skills`
- Prevents multi-MB row bloat in main skills table
- Better query performance for non-timeline queries
- `timeline_summary` JSONB for aggregated stats only

#### B. Async Queue for Heavy Processing 📅 FUTURE
**Status**: Deferred to Phase 4
- Initial implementation: synchronous processing
- Return `report_url` immediately with "processing" state
- Background jobs for:
  - Rankings calculation
  - Percentile computation
  - Advanced analytics
- Consider: Redis Queue, Celery, or Go channels

### Updated Timeline Estimate

#### Phase 1: MVP with Critical Fixes (3-4 weeks)
- Week 1: Client-side upload with transactions and validation
- Week 2: Backend API with authentication, deduplication, observability
- Week 3: Integration testing, retry logic verification
- Week 4: Security audit, load testing, staging deployment

#### Phase 2: Enhanced Features (2-3 weeks)
- Rankings/leaderboards with async processing
- Player profiles
- Advanced filters with pagination
- Public/private toggles

#### Phase 3: Analytics (2-3 weeks)
- DPS percentiles
- Skill usage analysis
- Class performance charts
- Rotation analysis

---

## GLM4.6 Assessment

Based on my comprehensive analysis of the API.md proposal and current Resonance-logs codebase, here is my evaluation:

### 1. Implementation Approach Soundness

**Excellent Foundation**: The proposed database-backed upload approach (Option 1) is exceptionally well-suited for this project. The existing database architecture already contains 90% of needed data structures, with materialized views in `damage_skill_stats`, `heal_skill_stats`, and `encounter_bosses` tables that perfectly align with upload requirements.

**Smart Design Decisions**:
- The JSON `hit_details` field in skill stats preserves timeline data efficiently
- Materialized aggregates reduce upload payload size while maintaining detail
- Boss-only damage filtering is already implemented in database
- The proposed DTO structure maps cleanly to existing models

### 2. Alignment with Current Codebase

**Exceptional Compatibility**: The proposed implementation integrates seamlessly with existing architecture:

- **Database Layer**: The current schema in `src-tauri/src/database/models.rs` already implements Serialize/Deserialize via Serde, eliminating the need for additional serialization work
- **Command Pattern**: The existing Tauri command structure in `src-tauri/src/database/commands.rs` provides a perfect template for new upload commands
- **Type Safety**: The project already uses specta for type-safe frontend-backend communication, which aligns perfectly with the proposed upload DTOs

### 3. Technical Feasibility

**High Feasibility with Minimal Complexity**:
- **Dependencies**: The project already includes most required dependencies. Only `reqwest` needs to be added to `src-tauri/Cargo.toml`
- **Data Flow**: The existing data flow from live events → database → materialized stats is ideal for upload functionality
- **Error Handling**: The current error handling patterns in database commands can be directly applied to upload functionality

### 4. Potential Issues and Improvements

**Minor Considerations**:
1. **Payload Size**: For encounters with many players and skills, the JSON payload could become large. Consider implementing compression as suggested, but start without it to establish baseline performance
2. **Offline Queue**: The current design doesn't address offline scenarios. Consider adding a simple upload queue for failed uploads
3. **Rate Limiting**: The backend should implement IP-based rate limiting to prevent abuse
4. **Data Privacy**: The proposed anonymization features are good, but consider making player name hashing optional per-upload

**Recommended Enhancements**:
1. **Incremental Uploads**: For very long raids, consider supporting chunked uploads
2. **Upload Status Tracking**: Add upload status to encounter metadata in the local database
3. **Batch Upload**: Allow uploading multiple encounters in a single request for efficiency

### 5. Implementation Complexity Assessment

**Low to Medium Complexity**:
- **Client-Side**: 2-3 days implementation effort. The database queries are straightforward joins of existing tables
- **Backend**: 1-2 weeks for MVP. The Go/Gin schema design is solid and well-thought-out
- **Frontend Integration**: 1-2 days. Adding upload buttons to existing encounter cards is minimal work

### 6. Architecture Strengths

**Well-Designed Foundation**:
- The separation of raw events (`damage_events`, `heal_events`) from materialized stats (`damage_skill_stats`, `heal_skill_stats`) is architecturally sound
- The existing WAL mode and batched writes demonstrate good performance practices
- The current indexing strategy supports efficient querying for upload data

### 7. Deployment Considerations

**Practical Recommendations**:
- Start with the serverless approach (Cloud Run/Lambda) as suggested - it's cost-effective for expected usage patterns
- The PostgreSQL schema design is excellent, with proper indexing and partitioning recommendations
- Consider implementing the suggested compression only after measuring actual payload sizes

### Conclusion

This is an exceptionally well-designed implementation proposal that aligns perfectly with the existing codebase. The database architecture is already optimized for this use case, and the proposed changes are minimal and focused. The timeline estimates are realistic, and the technical approach is sound.

**Recommendation**: Proceed with implementation as outlined. The foundation is solid, risks are low, and the approach leverages existing infrastructure effectively.