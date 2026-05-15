/**
 * logService — thin wrapper around @tauri-apps/plugin-log.
 *
 * All messages emitted here are routed through the Rust logging pipeline and
 * written to the same rotating log file as backend messages so that a single
 * log file tells the complete story of a session.
 *
 * Log file location (via Tauri's app_log_dir):
 *   Windows : %LOCALAPPDATA%\com.sparkinex.vocyl\logs\vocyl.log
 *   macOS   : ~/Library/Logs/vocyl/vocyl.log
 *   Linux   : ~/.local/share/com.sparkinex.vocyl/logs/vocyl.log
 */

import { info, warn, error, debug, trace } from '@tauri-apps/plugin-log';

/**
 * Structured logger object — mirrors the interface of `console` so call-sites
 * can switch between the two without further changes.
 *
 * Each message is prefixed with `[UI]` so backend and frontend messages are
 * easily distinguishable in the log file.
 */
export const logger = {
	info: (msg: string) => void info(`[UI] ${msg}`),
	warn: (msg: string) => void warn(`[UI] ${msg}`),
	error: (msg: string) => void error(`[UI] ${msg}`),
	debug: (msg: string) => void debug(`[UI] ${msg}`),
	trace: (msg: string) => void trace(`[UI] ${msg}`),
};
