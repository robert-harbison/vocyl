use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
pub struct SystemInfo {
    pub total_ram_mb: u64,
    pub cpu_count: usize,
}

/// Returns total installed RAM (MB) and logical CPU count.
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    let sys = System::new_all();
    SystemInfo {
        total_ram_mb: sys.total_memory() / 1_048_576,
        cpu_count: sys.cpus().len(),
    }
}

