use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::db;
use crate::error::AppError;
use crate::state::AppState;

// ─── API Keys ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_api_keys(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    _list_api_keys(&state).await.map_err(|e| e.to_string())
}

async fn _list_api_keys(state: &AppState) -> Result<Value, AppError> {
    let keys = db::api_keys::get_all_for_user(&state.db, state.local_user_id).await?;
    let sanitized: Vec<Value> = keys
        .iter()
        .map(|k| {
            let mut v = serde_json::to_value(k).unwrap_or_default();
            if let Some(key) = v.get_mut("api_key").and_then(|v| v.as_str().map(|s| s.to_string()))
            {
                v["api_key"] = Value::String(format!("{}...", &key[..key.len().min(10)]));
            }
            v
        })
        .collect();
    Ok(json!({ "apiKeys": sanitized }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub key_name: String,
}

#[tauri::command]
pub async fn create_api_key(
    state: tauri::State<'_, Arc<AppState>>,
    body: CreateApiKeyRequest,
) -> Result<Value, String> {
    _create_api_key(&state, body).await.map_err(|e| e.to_string())
}

async fn _create_api_key(state: &AppState, body: CreateApiKeyRequest) -> Result<Value, AppError> {
    let name = body.key_name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Key name is required".to_string()));
    }
    let key = db::api_keys::create(&state.db, state.local_user_id, name).await?;
    Ok(json!({ "success": true, "apiKey": key }))
}

#[tauri::command]
pub async fn delete_api_key(
    state: tauri::State<'_, Arc<AppState>>,
    key_id: i64,
) -> Result<Value, String> {
    _delete_api_key(&state, key_id).await.map_err(|e| e.to_string())
}

async fn _delete_api_key(state: &AppState, key_id: i64) -> Result<Value, AppError> {
    let deleted = db::api_keys::delete(&state.db, key_id, state.local_user_id).await?;
    if deleted {
        Ok(json!({ "success": true }))
    } else {
        Err(AppError::NotFound("API key not found".to_string()))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleRequest {
    pub is_active: bool,
}

#[tauri::command]
pub async fn toggle_api_key(
    state: tauri::State<'_, Arc<AppState>>,
    key_id: i64,
    is_active: bool,
) -> Result<Value, String> {
    _toggle_api_key(&state, key_id, is_active).await.map_err(|e| e.to_string())
}

async fn _toggle_api_key(state: &AppState, key_id: i64, is_active: bool) -> Result<Value, AppError> {
    let toggled = db::api_keys::toggle(&state.db, key_id, state.local_user_id, is_active).await?;
    if toggled {
        Ok(json!({ "success": true }))
    } else {
        Err(AppError::NotFound("API key not found".to_string()))
    }
}

// ─── Credentials ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn list_credentials(
    state: tauri::State<'_, Arc<AppState>>,
    cred_type: Option<String>,
) -> Result<Value, String> {
    _list_credentials(&state, cred_type.as_deref()).await.map_err(|e| e.to_string())
}

async fn _list_credentials(state: &AppState, cred_type: Option<&str>) -> Result<Value, AppError> {
    let creds = db::credentials::get_all(&state.db, state.local_user_id, cred_type).await?;
    Ok(json!({ "credentials": creds }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCredentialRequest {
    pub credential_name: String,
    pub credential_type: String,
    pub credential_value: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn create_credential(
    state: tauri::State<'_, Arc<AppState>>,
    body: CreateCredentialRequest,
) -> Result<Value, String> {
    _create_credential(&state, body).await.map_err(|e| e.to_string())
}

async fn _create_credential(state: &AppState, body: CreateCredentialRequest) -> Result<Value, AppError> {
    if body.credential_name.trim().is_empty() {
        return Err(AppError::BadRequest("Credential name is required".to_string()));
    }
    if body.credential_type.trim().is_empty() {
        return Err(AppError::BadRequest("Credential type is required".to_string()));
    }
    if body.credential_value.trim().is_empty() {
        return Err(AppError::BadRequest("Credential value is required".to_string()));
    }
    let cred = db::credentials::create(
        &state.db,
        state.local_user_id,
        body.credential_name.trim(),
        body.credential_type.trim(),
        body.credential_value.trim(),
        body.description.as_deref().map(|s| s.trim()),
    )
    .await?;
    Ok(json!({ "success": true, "credential": cred }))
}

#[tauri::command]
pub async fn delete_credential(
    state: tauri::State<'_, Arc<AppState>>,
    credential_id: i64,
) -> Result<Value, String> {
    _delete_credential(&state, credential_id).await.map_err(|e| e.to_string())
}

async fn _delete_credential(state: &AppState, credential_id: i64) -> Result<Value, AppError> {
    let deleted = db::credentials::delete(&state.db, credential_id, state.local_user_id).await?;
    if deleted {
        Ok(json!({ "success": true }))
    } else {
        Err(AppError::NotFound("Credential not found".to_string()))
    }
}

#[tauri::command]
pub async fn toggle_credential(
    state: tauri::State<'_, Arc<AppState>>,
    credential_id: i64,
    is_active: bool,
) -> Result<Value, String> {
    _toggle_credential(&state, credential_id, is_active).await.map_err(|e| e.to_string())
}

async fn _toggle_credential(state: &AppState, credential_id: i64, is_active: bool) -> Result<Value, AppError> {
    let toggled = db::credentials::toggle(&state.db, credential_id, state.local_user_id, is_active).await?;
    if toggled {
        Ok(json!({ "success": true }))
    } else {
        Err(AppError::NotFound("Credential not found".to_string()))
    }
}

// ─── Notification Preferences ────────────────────────────────────────────────

#[tauri::command]
pub async fn get_notification_preferences(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    _get_notification_preferences(&state).await.map_err(|e| e.to_string())
}

async fn _get_notification_preferences(state: &AppState) -> Result<Value, AppError> {
    let prefs = db::notifications::get_preferences(&state.db, state.local_user_id).await?;
    let prefs_value: Value = match prefs {
        Some(json_str) => serde_json::from_str(&json_str).unwrap_or(Value::Null),
        None => Value::Null,
    };
    Ok(json!({ "success": true, "preferences": prefs_value }))
}

#[tauri::command]
pub async fn update_notification_preferences(
    state: tauri::State<'_, Arc<AppState>>,
    body: Value,
) -> Result<Value, String> {
    _update_notification_preferences(&state, body).await.map_err(|e| e.to_string())
}

async fn _update_notification_preferences(state: &AppState, body: Value) -> Result<Value, AppError> {
    let prefs_json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());
    db::notifications::update_preferences(&state.db, state.local_user_id, &prefs_json).await?;
    Ok(json!({ "success": true, "preferences": body }))
}

// ─── Push Subscriptions ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_vapid_public_key(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Value, String> {
    _get_vapid_public_key(&state).await.map_err(|e| e.to_string())
}

async fn _get_vapid_public_key(state: &AppState) -> Result<Value, AppError> {
    let keys = db::push_subscriptions::get_or_create_vapid_keys(&state.db).await?;
    Ok(json!({ "publicKey": keys.0 }))
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

#[derive(Deserialize)]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

#[tauri::command]
pub async fn push_subscribe(
    state: tauri::State<'_, Arc<AppState>>,
    body: SubscribeRequest,
) -> Result<Value, String> {
    _push_subscribe(&state, body).await.map_err(|e| e.to_string())
}

async fn _push_subscribe(state: &AppState, body: SubscribeRequest) -> Result<Value, AppError> {
    if body.endpoint.is_empty() || body.keys.p256dh.is_empty() || body.keys.auth.is_empty() {
        return Err(AppError::BadRequest("Missing subscription fields".to_string()));
    }
    db::push_subscriptions::save(
        &state.db,
        state.local_user_id,
        &body.endpoint,
        &body.keys.p256dh,
        &body.keys.auth,
    )
    .await?;
    Ok(json!({ "success": true }))
}

#[derive(Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

#[tauri::command]
pub async fn push_unsubscribe(
    state: tauri::State<'_, Arc<AppState>>,
    body: UnsubscribeRequest,
) -> Result<Value, String> {
    _push_unsubscribe(&state, body).await.map_err(|e| e.to_string())
}

async fn _push_unsubscribe(state: &AppState, body: UnsubscribeRequest) -> Result<Value, AppError> {
    if body.endpoint.is_empty() {
        return Err(AppError::BadRequest("Missing endpoint".to_string()));
    }
    db::push_subscriptions::remove(&state.db, &body.endpoint).await?;
    Ok(json!({ "success": true }))
}
