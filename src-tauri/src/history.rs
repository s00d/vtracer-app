use crate::vtracer_service::VtracerSettings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const HISTORY_LIMIT: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStats {
    pub elapsed_ms: u64,
    pub path_count: Option<u32>,
    pub output_bytes: Option<u64>,
    pub input_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: u64,
    pub mode: String,
    pub input_paths: Vec<String>,
    pub output_paths: Vec<String>,
    pub settings: VtracerSettings,
    pub stats: HistoryStats,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedPreset {
    pub id: String,
    pub name: String,
    pub settings: VtracerSettings,
    pub created_at: u64,
}

pub fn ensure_data_dir(base: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(base).map_err(|e| format!("Could not create {base:?}: {e}"))?;
    Ok(base.to_path_buf())
}

fn history_path(base: &Path) -> PathBuf {
    base.join("history.json")
}

fn presets_path(base: &Path) -> PathBuf {
    base.join("presets.json")
}

fn app_settings_path(base: &Path) -> PathBuf {
    base.join("app-settings.json")
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<Vec<T>, String> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(path).map_err(|e| format!("Read error {path:?}: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(vec![]);
    }
    serde_json::from_str(&raw).map_err(|e| format!("Parse error {path:?}: {e}"))
}

fn write_json_file<T: Serialize>(path: &PathBuf, data: &[T]) -> Result<(), String> {
    let raw =
        serde_json::to_string_pretty(data).map_err(|e| format!("Serialization error: {e}"))?;
    fs::write(path, raw).map_err(|e| format!("Write error {path:?}: {e}"))
}

pub fn history_list(base: &Path) -> Result<Vec<HistoryEntry>, String> {
    read_json_file(&history_path(base))
}

pub fn history_add(base: &Path, entry: HistoryEntry) -> Result<HistoryEntry, String> {
    let path = history_path(base);
    let mut items = read_json_file::<HistoryEntry>(&path)?;
    items.insert(0, entry.clone());
    items.truncate(HISTORY_LIMIT);
    write_json_file(&path, &items)?;
    Ok(entry)
}

pub fn history_delete(base: &Path, id: String) -> Result<(), String> {
    let path = history_path(base);
    let mut items = read_json_file::<HistoryEntry>(&path)?;
    let len_before = items.len();
    items.retain(|e| e.id != id);
    if items.len() == len_before {
        return Err(format!("History entry not found: {id}"));
    }
    write_json_file(&path, &items)
}

pub fn history_clear(base: &Path) -> Result<(), String> {
    write_json_file(&history_path(base), &[] as &[HistoryEntry])
}

pub fn presets_list(base: &Path) -> Result<Vec<NamedPreset>, String> {
    read_json_file(&presets_path(base))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetSaveRequest {
    pub name: String,
    pub settings: VtracerSettings,
}

pub fn presets_save(base: &Path, request: PresetSaveRequest) -> Result<NamedPreset, String> {
    let path = presets_path(base);
    let mut items = read_json_file::<NamedPreset>(&path)?;
    let preset = NamedPreset {
        id: Uuid::new_v4().to_string(),
        name: request.name.trim().to_string(),
        settings: request.settings,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    };
    items.push(preset.clone());
    write_json_file(&path, &items)?;
    Ok(preset)
}

pub fn presets_delete(base: &Path, id: String) -> Result<(), String> {
    let path = presets_path(base);
    let mut items = read_json_file::<NamedPreset>(&path)?;
    let len_before = items.len();
    items.retain(|p| p.id != id);
    if items.len() == len_before {
        return Err(format!("Preset not found: {id}"));
    }
    write_json_file(&path, &items)
}

pub fn app_settings_load(base: &Path) -> Result<Option<VtracerSettings>, String> {
    let path = app_settings_path(base);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("Read error {path:?}: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&raw).map_err(|e| format!("Parse error {path:?}: {e}"))
}

pub fn app_settings_save(base: &Path, settings: VtracerSettings) -> Result<(), String> {
    let path = app_settings_path(base);
    let raw =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("Serialization error: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("Write error {path:?}: {e}"))
}
