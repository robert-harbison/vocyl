import { logger } from '@services/logService';

export function installGlobalErrorHandlers(): () => void {
	const controller = new AbortController();
	const { signal } = controller;

	window.addEventListener(
		'error',
		(event: ErrorEvent) => {
			const message = event.message satisfies string;
			const filename = event.filename satisfies string;
			const lineno = String(event.lineno);
			const colno = String(event.colno);
			const stack =
				event.error instanceof Error && event.error.stack ? `\n${event.error.stack}` : '';
			logger.error(`[GlobalError] ${message} (${filename}:${lineno}:${colno})${stack}`);
		},
		{ signal },
	);

	window.addEventListener(
		'unhandledrejection',
		(event: PromiseRejectionEvent) => {
			const reason: unknown = event.reason;
			let msg: string;
			if (reason instanceof Error) {
				msg = `${reason.name}: ${reason.message}`;
				if (reason.stack) msg += `\n${reason.stack}`;
			} else {
				msg = String(reason);
			}
			logger.error(`[UnhandledRejection] ${msg}`);
		},
		{ signal },
	);

	return () => {
		controller.abort();
	};
}
