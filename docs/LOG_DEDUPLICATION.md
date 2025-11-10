# Server-Side Encounter Deduplication

## Overview

The server now implements **cross-user deduplication** with **fuzzy matching** for uploaded encounters. This prevents multiple uploads of the same encounter from different players (POVs) from creating duplicate database entries.

## How It Works

### 1. Fingerprinting

When an encounter is uploaded, the server computes a **deterministic fingerprint** (SHA256 hash) based on:
- **Scene** (SceneID or SceneName)
- **Boss names** (sorted, lowercased)
- **Player set** (sorted ActorIDs with damage percentages rounded to 2 decimals)
- **Attempt count**
- **Start time bucket** (30-second buckets to distinguish separate runs)

The fingerprint is **order-independent** and **POV-independent**, meaning:
- Same encounter uploaded by different players → same fingerprint
- Different actor ordering in upload → same fingerprint
- Different boss ordering → same fingerprint

### 2. Player Set Hashing

For efficient fuzzy matching, the server also computes a **player set hash** (SHA256 of sorted ActorIDs only). This enables fast candidate lookups when searching for potential duplicates.

### 3. Duplicate Detection Flow

When an encounter is uploaded:

1. **Compute fingerprint** and player set hash server-side
2. **Exact duplicate check** (global scope, cross-user):
   - Check if fingerprint exists in database
   - Also check client-provided `sourceHash` for backward compatibility
   - If found → return existing encounter ID, skip insertion
3. **Fuzzy duplicate check** (if no exact match):
   - Query candidates with same `player_set_hash` (fast lookup)
   - For each candidate, compute similarity metrics:
     - Scene match (SceneID or SceneName)
     - Boss names match (sorted comparison)
     - Player set match (same ActorIDs)
     - Per-player damage L1 norm ≤ 5%
     - Total damage difference ≤ 3%
     - Start time delta ≤ 30 seconds
     - Attempt count match
   - If all criteria pass → treat as duplicate, return existing encounter ID
4. **No duplicate found** → insert new encounter with fingerprint and player_set_hash

### 4. Race Condition Handling

The database has a **unique index** on `fingerprint` to prevent concurrent uploads from creating duplicates. If a unique constraint violation occurs:
- Server catches the error
- Re-queries for the existing encounter
- Returns the existing encounter ID instead of failing

## Configuration

Default thresholds (hardcoded in `lib.DefaultDedupeConfig()`):
- **Start time bucket**: 30 seconds
- **Damage L1 threshold**: 0.05 (5%)
- **Total damage % difference**: 0.03 (3%)
- **Start time delta**: 30 seconds

These values are optimized to:
- **Separate distinct runs** (30s start time bucket distinguishes back-to-back runs)
- **Allow minor stat differences** (5% per-player damage tolerance handles rounding/POV differences)
- **Strict enough to avoid false positives** (3% total damage + 30s time delta)

## Database Schema

New columns in `encounters` table:
```sql
fingerprint VARCHAR(64)       -- SHA256 hex of canonical encounter data
player_set_hash VARCHAR(64)   -- SHA256 hex of sorted player ActorIDs
```

New indexes:
```sql
idx_fingerprint              -- Fast exact duplicate lookups
uniq_fingerprint             -- Enforce global uniqueness (WHERE fingerprint IS NOT NULL)
idx_player_set_hash          -- Fast fuzzy candidate lookups
```

## Migration

### Development (AUTO_MIGRATE=true)
GORM AutoMigrate will automatically add the columns and indexes.

### Production
Run the SQL migration:
```bash
psql -d resonance -f server/migrations/20251110_add_encounter_fingerprint.sql
```

### Backfill Existing Encounters (Optional)
After migration, you can backfill fingerprints for historical encounters:
```sql
-- Note: Requires implementing a batch job that:
-- 1. Loads encounters with preloaded Players, Bosses, Attempts
-- 2. Computes fingerprint using lib.ComputeEncounterFingerprint
-- 3. Updates fingerprint and player_set_hash columns
UPDATE encounters SET
  fingerprint = compute_fingerprint(...),
  player_set_hash = compute_player_set_hash(...)
WHERE fingerprint IS NULL;
```

## API Behavior

### POST /api/v1/upload
- Accepts `EncounterIn[]` with optional client-provided `sourceHash`
- Server computes fingerprint for each encounter
- Returns `UploadEncountersResponse` with:
  - `ingested`: count of encounters processed
  - `ids`: encounter IDs (new or existing if duplicate)
- If duplicate detected (exact or fuzzy), returns existing encounter ID without creating new row

### POST /api/v1/upload/check
- Accepts `{ hashes: string[] }` (client-provided hashes)
- Checks both `source_hash` and `fingerprint` columns (global scope)
- Returns:
  - `duplicates`: array of `{ hash, encounterId }` for found hashes
  - `missing`: array of hashes not found in database

## Example Scenarios

### Scenario 1: Same encounter, different POVs
- **User1** uploads encounter with LocalPlayerID=1001
- **User2** uploads same encounter with LocalPlayerID=1002
- **Result**: User2's upload detects exact fingerprint match → returns User1's encounter ID

### Scenario 2: Multiple runs of same dungeon
- **User1** uploads run at 12:00:00 (fingerprint bucket: 12:00:00-12:00:30)
- **User1** uploads another run at 12:01:00 (fingerprint bucket: 12:01:00-12:01:30)
- **Result**: Different fingerprints due to different time buckets → both inserted as separate encounters

### Scenario 3: Fuzzy match (near-duplicate)
- **User1** uploads encounter with Player1=3000dmg, Player2=4000dmg
- **User2** uploads encounter with Player1=3050dmg, Player2=3950dmg (1.7% difference per player)
- Scene, boss, players, time all match within thresholds
- **Result**: Fuzzy duplicate detected → returns User1's encounter ID

### Scenario 4: Concurrent uploads (race condition)
- **User1** and **User2** upload same encounter simultaneously
- Both transactions compute same fingerprint
- Both attempt to insert
- **Result**: One succeeds, other gets unique constraint violation → re-queries and returns first one's encounter ID

## Testing

Unit tests in `server/lib/dedupe_test.go` validate:
- Fingerprint determinism
- Order independence (actors, bosses)
- Time bucketing behavior
- Player set hash stability

To run tests:
```bash
cd server
go test ./lib -v
```

## Performance Considerations

### Exact Duplicate Check
- **Fast**: Single indexed lookup on `fingerprint` (64-char string)
- **Cost**: O(1) with B-tree index

### Fuzzy Duplicate Check
- **Candidate query**: Indexed lookup on `player_set_hash` (fast)
- **Similarity computation**: O(N) where N = number of candidates with same player set
- **Typical case**: 0-5 candidates per player set
- **Worst case**: If many encounters share same player set, consider adding composite index `(scene_id, player_set_hash)` or limiting candidate count

### Preloading
Fuzzy check preloads:
- `Players` (ActorEncounterStats)
- `Bosses` (EncounterBosses)
- `Attempts`

This adds N+1 queries per candidate. For typical workloads (1-3 candidates), this is acceptable.

## Future Enhancements

### Configurable Thresholds
Add environment variables to tune fuzzy matching:
```env
DEDUPE_START_TIME_BUCKET_SEC=30
DEDUPE_DAMAGE_L1_THRESHOLD=0.05
DEDUPE_TOTAL_DAMAGE_PCT=0.03
DEDUPE_START_TIME_DELTA_SEC=30
```

### Backfill Job
Implement CLI command or background job:
```bash
go run cmd/backfill/main.go --batch-size=1000
```

### Partial Match Reporting
Instead of auto-skipping fuzzy duplicates, return them as "possible duplicates" for client review:
```json
{
  "ingested": 1,
  "ids": [123],
  "possibleDuplicates": [
    {"encounterId": 456, "similarity": 0.92}
  ]
}
```

### Cross-Scene Deduplication
Currently scene must match. Could relax to detect "same encounter, wrong scene metadata" by checking boss+players only.

## Known Limitations

1. **Clock skew**: If client clocks differ by >30s, same encounter may not match time bucket. Mitigation: increase bucket size or rely more on fuzzy matching.
2. **Partial POVs**: If one upload has subset of players, fingerprints differ. Fuzzy matching won't detect this as player sets differ.
3. **Boss name variations**: Minor typos in boss names prevent matching. Mitigation: normalize boss names server-side.
4. **Damage rounding**: Very small damage differences (<0.01%) may accumulate across many players. Current 5% L1 threshold should handle this.

## Questions / Troubleshooting

### Why is my upload being marked as duplicate?
Check:
1. Fingerprint collision (unlikely with SHA256)
2. Time bucket overlap (encounters within 30s of each other)
3. Fuzzy match false positive (adjust thresholds if needed)

### Can I disable fuzzy matching?
Not currently configurable. To disable, comment out the fuzzy matching block in `UploadEncounters`.

### How do I force upload even if duplicate?
Send a unique `sourceHash` from client that doesn't match any existing encounter. Server will still compute fingerprint but won't find source_hash match.

### Performance issues with fuzzy matching?
1. Add composite index: `CREATE INDEX idx_scene_player_set ON encounters (scene_id, player_set_hash);`
2. Limit candidate count: add `.Limit(10)` to candidate query
3. Disable preloading and compute similarity on-demand for top candidates only
