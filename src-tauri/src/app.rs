// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn exit_app<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    log::info!("User requested application exit");
    app.exit(0);
}
