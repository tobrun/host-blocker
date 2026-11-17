//! Thin `#[tauri::command]` layer: every command clones the shared [`App`],
//! runs the blocking work on Tauri's blocking pool, and returns either the
//! new [`UiState`] or a serialisable [`AppError`].

use std::sync::Arc;

use tauri::State;

use crate::app::{App, UiState};
use crate::baseline::stevenblack::Extension;
use crate::domain::mode::BlockingMode;
use crate::error::AppResult;

async fn call<F>(app: State<'_, Arc<App>>, f: F) -> AppResult<UiState>
where
    F: FnOnce(Arc<App>) -> AppResult<UiState> + Send + 'static,
{
    let app = app.inner().clone();
    tauri::async_runtime::spawn_blocking(move || f(app))
        .await
        .map_err(|e| crate::error::AppError::Data(format!("task panicked: {e}")))?
}

#[tauri::command]
pub async fn get_state(app: State<'_, Arc<App>>) -> AppResult<UiState> {
    call(app, |app| app.get_state()).await
}

#[tauri::command]
pub async fn add_site(app: State<'_, Arc<App>>, input: String) -> AppResult<UiState> {
    call(app, move |app| app.add_site(&input)).await
}

#[tauri::command]
pub async fn remove_site(app: State<'_, Arc<App>>, input: String) -> AppResult<UiState> {
    call(app, move |app| app.remove_site(&input)).await
}

#[tauri::command]
pub async fn import_baseline(
    app: State<'_, Arc<App>>,
    extensions: Vec<Extension>,
) -> AppResult<UiState> {
    call(app, move |app| app.import_baseline(&extensions)).await
}

#[tauri::command]
pub async fn update_baseline(app: State<'_, Arc<App>>) -> AppResult<UiState> {
    call(app, |app| app.update_baseline()).await
}

#[tauri::command]
pub async fn remove_baseline(app: State<'_, Arc<App>>) -> AppResult<UiState> {
    call(app, |app| app.remove_baseline()).await
}

#[tauri::command]
pub async fn set_mode(app: State<'_, Arc<App>>, mode: BlockingMode) -> AppResult<UiState> {
    call(app, move |app| app.set_mode(mode)).await
}

#[tauri::command]
pub async fn remove_all_entries(app: State<'_, Arc<App>>) -> AppResult<UiState> {
    call(app, |app| app.remove_all_entries()).await
}
