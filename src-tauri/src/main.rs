// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // ── Panic hook ────────────────────────────────────────────────────────────
    // Install before anything else so that crashes anywhere — including inside
    // Tauri's setup — are captured. `log::error!` only works after
    // tauri-plugin-log is registered; `eprintln!` is the unconditional fallback
    // for panics that fire before the log subscriber is up.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());

        let msg = format!("PANIC at {}: {}", location, payload);

        // Try the structured logger first (works once tauri-plugin-log is up).
        log::error!("{}", msg);

        // Always write to stderr — catches pre-init panics and surfaces in any
        // terminal / OS crash reporter regardless of log state.
        eprintln!("[vocyl PANIC] {}", msg);

        // Give the async log writer a brief moment to flush the file buffer
        // before the default hook terminates the process.
        std::thread::sleep(std::time::Duration::from_millis(150));

        // Invoke the original hook so Rust still prints the backtrace and
        // calls the platform's abort/exit routine.
        default_hook(info);
    }));

    vocyl_lib::run()
}
