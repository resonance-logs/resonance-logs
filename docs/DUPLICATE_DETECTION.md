# Duplicate Log Detection Feature

## Overview

This feature prevents duplicate combat log uploads by computing a deterministic hash for each encounter and checking with the server before uploading. This saves bandwidth, prevents database bloat, and improves user experience.

## Architecture

### Client-Side (resonance-logs)

1. **Hash Computation** (`src-tauri/src/uploader/mod.rs`)
   - Computes SHA-256 hash of canonical encounter data
   - Canonical fields: `startedAtMs`, `endedAtMs`, `localPlayerId`, `sceneId`, `sceneName`, sorted attempts, sorted actor IDs
   - Hash is deterministic (same encounter data → same hash every time)
   - Uses `sha2` and `hex` crates

2. **Preflight Check** (before each batch upload)
   - POST to `/api/v1/upload/check` with array of hashes
   - Server returns which hashes are duplicates and which are missing
   - Client filters out duplicates from the batch before uploading
   - Emits UI progress events for skipped duplicates

3. **Source Hash in Upload**
   - `UploadEncounterIn` struct includes `source_hash` field
   - Sent to server with each encounter for server-side deduplication

### Server-Side (resonance-website)

1. **Database Schema** (`server/models/encounter.go`)
   - New field: `source_hash` (nullable, varchar(64))
   - Composite index: `idx_user_source_hash` on `(user_id, source_hash)`
   - Fast duplicate lookups per user

2. **Preflight Endpoint** (`/api/v1/upload/check`)
   - Handler: `CheckDuplicates` in `server/controller/upload/upload.go`
   - Accepts: `{ "hashes": ["hash1", "hash2", ...] }` (max 50)
   - Returns: `{ "duplicates": [{"hash": "...", "encounterId": 123}], "missing": ["hash3"] }`
   - Protected by `EitherAuth` middleware (API key or session cookie)

3. **Upload Deduplication** (`/api/v1/upload/`)
   - Before inserting each encounter, queries for existing `(user_id, source_hash)` match
   - If found: skips insertion, returns existing encounter ID
   - If not found: creates new encounter with `source_hash`
   - Server-side dedupe is authoritative (works even if client skips preflight)

## API Endpoints

### POST /api/v1/upload/check

**Purpose**: Preflight check to identify which encounters are already uploaded

**Authentication**: Requires `X-Api-Key` header or session cookie

**Request**:
```json
{
  "hashes": ["abc123...", "def456..."]
}
```

**Response**:
```json
{
  "duplicates": [
    {
      "hash": "abc123...",
      "encounterId": 12345
    }
  ],
  "missing": ["def456..."]
}
```

**Error Codes**:
- `400`: Invalid payload or too many hashes (max 50)
- `401`: Unauthorized (missing or invalid API key)
- `500`: Database error

### POST /api/v1/upload/ (Updated)

**Changes**:
- Accepts optional `sourceHash` field in each encounter
- Checks for duplicates before inserting
- Returns existing encounter ID if duplicate found

**Request** (per encounter):
```json
{
  "encounters": [{
    "startedAtMs": 1234567890000,
    "sourceHash": "abc123...",
    ...other fields...
  }]
}
```

**Response** (unchanged):
```json
{
  "ingested": 1,
  "ids": [12345]
}
```

Note: `ingested` counts the number of encounters in the request, not the number of new DB rows created.

## Hash Computation Details

### Canonical Fields (Client)

The hash includes these fields in this order:
1. `startedAtMs` (i64)
2. `endedAtMs` (Option<i64>)
3. `localPlayerId` (Option<i64>)
4. `sceneId` (Option<i32>)
5. `sceneName` (Option<String>)
6. `attempts` (sorted by `attemptIndex`, includes `attemptIndex`, `startedAtMs`, `endedAtMs`)
7. `actorIds` (sorted array of actor IDs from `actorEncounterStats`)

### Implementation (Rust)

```rust
fn compute_encounter_hash(encounter: &UploadEncounterIn) -> String {
    // Sort attempts and actors for determinism
    let mut sorted_attempts = encounter.attempts.clone();
    sorted_attempts.sort_by_key(|a| a.attempt_index);

    let mut actor_ids: Vec<i64> = encounter.actor_encounter_stats
        .iter()
        .map(|s| s.actor_id)
        .collect();
    actor_ids.sort();

    // Build canonical JSON
    let canonical = json!({
        "startedAtMs": encounter.started_at_ms,
        "endedAtMs": encounter.ended_at_ms,
        "localPlayerId": encounter.local_player_id,
        "sceneId": encounter.scene_id,
        "sceneName": encounter.scene_name,
        "attempts": attempt_values,
        "actorIds": actor_ids,
    });

    // Hash the canonical JSON string
    let canonical_str = serde_json::to_string(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical_str.as_bytes());
    hex::encode(hasher.finalize())
}
```

## Database Migration

### Development (Auto-Migrate)

Set `AUTO_MIGRATE=true` environment variable. GORM will add the column and index automatically.

### Production (Manual Migration)

```sql
-- Add source_hash column
ALTER TABLE encounters ADD COLUMN source_hash VARCHAR(64);

-- Create composite index for fast lookups
CREATE INDEX idx_user_source_hash ON encounters (user_id, source_hash);
```

### Backfill (Optional)

Existing encounters will have `NULL` source_hash. Options:
1. Leave as NULL (new uploads won't match old encounters)
2. Compute hashes for historical data (complex, requires reconstruct canonical representation from DB)
3. Accept that only new uploads benefit from deduplication

Recommended: Option 1 (leave as NULL). Historical duplicates are rare.

## Testing

### Unit Tests

**Client (Rust)**:
- Test hash determinism: same input → same hash
- Test hash uniqueness: different inputs → different hashes
- Test sorted attempt/actor handling

**Server (Go)**:
- Test CheckDuplicates handler with various hash arrays
- Test UploadEncounters skips duplicates correctly
- Test concurrent uploads with same source_hash (unique constraint handling)

### Integration Tests

1. Upload encounter with source_hash
2. Call /check endpoint → should return as duplicate
3. Re-upload same encounter → should return existing ID
4. Upload batch with mix of new and duplicate → correct IDs returned
5. Verify database has no duplicate rows for same source_hash + user_id

See `test_duplicate_api.ps1` for manual integration test script.

## Performance Considerations

- **Index**: `(user_id, source_hash)` composite index enables fast lookups (O(log n))
- **Preflight batching**: Checking 10 hashes in one request vs 10 separate requests
- **Client-side filtering**: Reduces upload bandwidth by skipping duplicates before POST
- **Server-side deduplication**: Ensures correctness even if client skips preflight

## Security

- **Per-user isolation**: Hashes are checked per user (user_id in WHERE clause)
- **No hash collisions**: SHA-256 is cryptographically secure (collision probability negligible)
- **Authentication**: Both /check and /upload require valid API key or session
- **Rate limiting**: Consider adding rate limits to /check endpoint to prevent abuse

## Error Handling

**Client**:
- If preflight /check fails: proceed with full batch upload (server will dedupe)
- If upload fails: retry logic unchanged
- Emit clear UI messages for skipped duplicates vs upload errors

**Server**:
- If duplicate check query fails: return 500 (don't proceed with insert)
- If insert fails due to unique constraint: perform SELECT to get existing ID
- Return clear error messages with HTTP status codes

## Future Enhancements

1. **Batch deduplication**: Check all encounter hashes in DB in single query (already implemented)
2. **Partial match detection**: Detect encounters with same time/scene but different hash (corrupted data)
3. **Hash versioning**: Include schema version in hash computation for forward compatibility
4. **Backfill tool**: CLI tool to compute hashes for historical encounters
5. **Analytics**: Track duplicate upload attempts to identify common user workflows

## Dependencies

**Client (Rust)**:
- `sha2 = "0.10"` - SHA-256 hashing
- `hex = "0.4"` - Hex encoding for hash output

**Server (Go)**:
- Standard library only (no new dependencies)
- GORM for database operations (existing)

## References

- Server controller: `server/controller/upload/upload.go`
- Server model: `server/models/encounter.go`
- Server routes: `server/routes/groups/upload.go`
- Client uploader: `resonance-logs/src-tauri/src/uploader/mod.rs`
- Migration: `server/migrations/migrate.go`
- Test script: `server/test_duplicate_api.ps1`
