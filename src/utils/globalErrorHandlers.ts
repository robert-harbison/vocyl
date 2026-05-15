/**
 * installGlobalErrorHandlers
 *
 * Registers window-level listeners that catch every uncaught JavaScript
 * exception and every unhandled Promise rejection, then forwards them to
 * the structured logger so they appear in the app log file alongside
 * Rust backend messages.
 *
 * Call this once at the very top of each entry-point (main.tsx, overlay.tsx)
 * BEFORE rendering the React tree.
 */

import { logger } from '@services/logService';

export function installGlobalErrorHandlers(): void {
	// ── Uncaught synchronous errors ───────────────────────────────────────────
	window.addEventListener('error', (event: ErrorEvent) => {
		const message = String(event.message);
		const filename = String(event.filename);
		const lineno = String(event.lineno);
		const colno = String(event.colno);
		const stack =
			event.error instanceof Error && event.error.stack ? `\n${event.error.stack}` : '';
		logger.error(`[GlobalError] ${message} (${filename}:${lineno}:${colno})${stack}`);
		// Do NOT call event.preventDefault() — let the browser still surface
		// the error in DevTools during development.
	});

	// ── Unhandled Promise rejections ──────────────────────────────────────────
	window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
		const reason: unknown = event.reason;
		let msg: string;
		if (reason instanceof Error) {
			msg = `${reason.name}: ${reason.message}`;
			if (reason.stack) msg += `\n${reason.stack}`;
		} else {
			msg = String(reason);
		}
		logger.error(`[UnhandledRejection] ${msg}`);
	});
}
