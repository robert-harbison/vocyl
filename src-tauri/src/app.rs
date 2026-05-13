// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn exit_app<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    app.exit(0);
}
