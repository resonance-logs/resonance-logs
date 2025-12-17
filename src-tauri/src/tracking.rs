//! Simple telemetry module to track application updates.
//!
//! On first run or after an update, sends an anonymous ping to the server.
//! Stores client_id and last_reported_version in the local SQLite database.

use log::{info, warn};
use reqwest::Client;
use serde::Serialize;
use uuid::Uuid;

use crate::database::establish_connection;
use crate::database::ensure_migrations_on_conn;
use crate::database::schema::app_config;
use diesel::prelude::*;

const API_URLS: &[&str] = &["https://api.bpsr.app/api/v1/tracking/app-update"];

const KEY_CLIENT_ID: &str = "tracking_client_id";
const KEY_LAST_REPORTED_VERSION: &str = "tracking_last_reported_version";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackingPayload {
    client_id: String,
    version: String,
    platform: String,
}

/// Get a config value from the app_config table
fn get_config_value(conn: &mut diesel::sqlite::SqliteConnection, key: &str) -> Option<String> {
    use app_config::dsl;
    dsl::app_config
        .filter(dsl::key.eq(key))
        .select(dsl::value)
        .first::<String>(conn)
        .ok()
}

/// Set a config value in the app_config table (upsert)
fn set_config_value(
    conn: &mut diesel::sqlite::SqliteConnection,
    key: &str,
    value: &str,
) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO app_config (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(value)
    .execute(conn)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Checks if the version changed and sends a tracking ping to the server.
/// Call this from the application setup phase.
pub async fn track_update(app_handle: tauri::AppHandle) {
    let current_version = app_handle.package_info().version.to_string();

    // Load config from database in blocking context
    let (client_id, last_reported_version) = match tauri::async_runtime::spawn_blocking(move || {
        let mut conn = match establish_connection() {
            Ok(c) => c,
            Err(e) => {
                warn!("tracking: failed to connect to db: {}", e);
                return None;
            }
        };

        // Make sure the schema exists before touching app_config.
        if let Err(e) = ensure_migrations_on_conn(&mut conn) {
            warn!("tracking: failed to run migrations: {}", e);
            return None;
        }

        let client_id = get_config_value(&mut conn, KEY_CLIENT_ID)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let last_version =
            get_config_value(&mut conn, KEY_LAST_REPORTED_VERSION).unwrap_or_default();

        // Ensure client_id is persisted
        if get_config_value(&mut conn, KEY_CLIENT_ID).is_none() {
            let _ = set_config_value(&mut conn, KEY_CLIENT_ID, &client_id);
        }

        Some((client_id, last_version))
    })
    .await
    {
        Ok(Some((cid, lv))) => (cid, lv),
        Ok(None) => return,
        Err(e) => {
            warn!("tracking: failed to load config: {}", e);
            return;
        }
    };

    if last_reported_version == current_version {
        info!("tracking: version {} already reported", current_version);
        return;
    }

    info!(
        "tracking: version changed from '{}' to '{}', sending update",
        last_reported_version, current_version
    );

    let payload = TrackingPayload {
        client_id: client_id.clone(),
        version: current_version.clone(),
        platform: std::env::consts::OS.to_string(),
    };

    let client = Client::new();
    let mut success = false;

    for url in API_URLS {
        match client.post(*url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("tracking: successfully reported to {}", url);
                success = true;
                break;
            }
            Ok(resp) => {
                warn!("tracking: server {} returned {}", url, resp.status());
            }
            Err(e) => {
                warn!("tracking: failed to reach {}: {}", url, e);
            }
        }
    }

    if success {
        let version_to_save = current_version;
        if let Err(e) = tauri::async_runtime::spawn_blocking(move || {
            let mut conn = match establish_connection() {
                Ok(c) => c,
                Err(e) => return Err(e.to_string()),
            };
            set_config_value(&mut conn, KEY_LAST_REPORTED_VERSION, &version_to_save)
        })
        .await
        {
            warn!("tracking: failed to save config: {}", e);
        }
    }
}
