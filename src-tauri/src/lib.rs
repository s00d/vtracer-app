mod history;
mod svg_postprocess;
mod vtracer_service;

use history::{HistoryEntry, NamedPreset, PresetSaveRequest};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use vtracer_service::{BatchRequest, ConvertRequest, PreviewRequest, SaveSvgRequest, VtracerSettings};

struct BatchRuntime {
    cancel_flag: Arc<AtomicBool>,
}

struct AppState {
    batch: Mutex<Option<BatchRuntime>>,
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("vtracer-app");
    history::ensure_data_dir(&dir)?;
    Ok(dir)
}

#[tauri::command]
async fn convert_to_svg(
    request: ConvertRequest,
) -> Result<vtracer_service::ConvertResponse, String> {
    tauri::async_runtime::spawn_blocking(move || vtracer_service::convert_to_svg(request))
        .await
        .map_err(|err| format!("Conversion task was interrupted: {err}"))?
}

#[tauri::command]
async fn convert_preview(
    request: PreviewRequest,
) -> Result<vtracer_service::PreviewResponse, String> {
    tauri::async_runtime::spawn_blocking(move || vtracer_service::convert_preview(request))
        .await
        .map_err(|err| format!("Preview task was interrupted: {err}"))?
}

#[tauri::command]
async fn convert_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BatchRequest,
) -> Result<vtracer_service::BatchResponse, String> {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut guard = state.batch.lock().map_err(|e| e.to_string())?;
        *guard = Some(BatchRuntime {
            cancel_flag: cancel_flag.clone(),
        });
    }

    let app_handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        vtracer_service::convert_batch(request, cancel_flag, |index, total, path, status, message| {
            let _ = app_handle.emit(
                "batch-progress",
                serde_json::json!({
                    "index": index,
                    "total": total,
                    "path": path,
                    "status": status,
                    "message": message,
                }),
            );
        })
    })
    .await
    .map_err(|err| format!("Batch task was interrupted: {err}"))?;

    {
        let mut guard = state.batch.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }

    Ok(result)
}

#[tauri::command]
fn cancel_batch(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state.batch.lock().map_err(|e| e.to_string())?;
    if let Some(runtime) = guard.as_ref() {
        runtime.cancel_flag.store(true, Ordering::SeqCst);
        Ok(())
    } else {
        Err("No active batch conversion".to_string())
    }
}

#[tauri::command]
async fn get_image_info(input_path: String) -> Result<vtracer_service::ImageInfo, String> {
    tauri::async_runtime::spawn_blocking(move || vtracer_service::get_image_info(input_path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn read_image_data_url(
    input_path: String,
    trim_empty: Option<bool>,
) -> Result<vtracer_service::ImageDataUrlResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        vtracer_service::read_image_data_url(input_path, trim_empty)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn scan_images_in_directory(dir_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        vtracer_service::scan_images_in_directory(dir_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn save_svg_as(
    request: SaveSvgRequest,
) -> Result<vtracer_service::SaveSvgResponse, String> {
    vtracer_service::save_svg_as(request)
}

#[tauri::command]
fn history_list(app: AppHandle) -> Result<Vec<HistoryEntry>, String> {
    history::history_list(&data_dir(&app)?)
}

#[tauri::command]
fn history_add(app: AppHandle, entry: HistoryEntry) -> Result<HistoryEntry, String> {
    history::history_add(&data_dir(&app)?, entry)
}

#[tauri::command]
fn history_delete(app: AppHandle, id: String) -> Result<(), String> {
    history::history_delete(&data_dir(&app)?, id)
}

#[tauri::command]
fn history_clear(app: AppHandle) -> Result<(), String> {
    history::history_clear(&data_dir(&app)?)
}

#[tauri::command]
fn presets_list(app: AppHandle) -> Result<Vec<NamedPreset>, String> {
    history::presets_list(&data_dir(&app)?)
}

#[tauri::command]
fn presets_save(app: AppHandle, request: PresetSaveRequest) -> Result<NamedPreset, String> {
    history::presets_save(&data_dir(&app)?, request)
}

#[tauri::command]
fn presets_delete(app: AppHandle, id: String) -> Result<(), String> {
    history::presets_delete(&data_dir(&app)?, id)
}

#[tauri::command]
fn app_settings_load(app: AppHandle) -> Result<Option<VtracerSettings>, String> {
    history::app_settings_load(&data_dir(&app)?)
}

#[tauri::command]
fn app_settings_save(app: AppHandle, settings: VtracerSettings) -> Result<(), String> {
    history::app_settings_save(&data_dir(&app)?, settings)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            batch: Mutex::new(None),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            convert_to_svg,
            convert_preview,
            convert_batch,
            cancel_batch,
            get_image_info,
            read_image_data_url,
            scan_images_in_directory,
            save_svg_as,
            history_list,
            history_add,
            history_delete,
            history_clear,
            presets_list,
            presets_save,
            presets_delete,
            app_settings_load,
            app_settings_save,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
