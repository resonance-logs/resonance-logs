/// Module upload functionality for syncing modules to resonance-website API
/// Handles HTTP communication, retries, and error handling

use crate::module_extractor::types::{
    ImportModulesRequest, ImportModulesResponse, ModuleImportData, ModuleInfo,
};
use log::{debug, error, info, warn};
use reqwest::Client;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

/// Upload modules to the module optimizer API
///
/// # Arguments
/// * `modules` - List of modules to upload
/// * `api_key` - API key for authentication
/// * `base_url` - Base URL of the API (e.g., "http://localhost:8080/api/v1")
///
/// # Returns
/// Result with ImportModulesResponse or error message
pub async fn upload_modules(
    modules: Vec<ModuleInfo>,
    api_key: &str,
    base_url: &str,
) -> Result<ImportModulesResponse, String> {
    if modules.is_empty() {
        return Ok(ImportModulesResponse {
            summary: crate::module_extractor::types::ImportSummary {
                added: 0,
                updated: 0,
                errors: 0,
            },
            errors: vec![],
        });
    }

    debug!("Uploading {} modules to {}", modules.len(), base_url);

    // Convert ModuleInfo to API format
    let module_data: Vec<ModuleImportData> = modules.into_iter().map(|m| m.into()).collect();

    let request_body = ImportModulesRequest {
        version: "1.0".to_string(),
        modules: module_data,
    };

    // Create HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = format!("{}/module-optimizer/modules/import", base_url.trim_end_matches('/'));

    // Retry logic
    let mut last_error = String::new();
    for attempt in 1..=MAX_RETRIES {
        match perform_upload(&client, &url, api_key, &request_body).await {
            Ok(response) => {
                info!(
                    "Module upload succeeded: added={}, updated={}, errors={}",
                    response.summary.added, response.summary.updated, response.summary.errors
                );
                return Ok(response);
            }
            Err(e) => {
                last_error = e.clone();
                warn!("Module upload attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);

                if attempt < MAX_RETRIES {
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
                }
            }
        }
    }

    error!("Module upload failed after {} retries: {}", MAX_RETRIES, last_error);
    Err(last_error)
}

/// Perform a single upload attempt
async fn perform_upload(
    client: &Client,
    url: &str,
    api_key: &str,
    request_body: &ImportModulesRequest,
) -> Result<ImportModulesResponse, String> {
    let response = client
        .post(url)
        .header("X-Api-Key", api_key)
        .header("Content-Type", "application/json")
        .json(request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();

    if status.is_success() {
        response
            .json::<ImportModulesResponse>()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {}", e))
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Server returned {}: {}", status, error_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_empty_modules() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = upload_modules(vec![], "test-key", "http://localhost:8080/api/v1").await;
            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.summary.added, 0);
        });
    }
}
