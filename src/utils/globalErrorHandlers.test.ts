import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { installGlobalErrorHandlers } from './globalErrorHandlers';

vi.mock('@services/logService');

import { logger } from '@services/logService';

const mockLoggerError = vi.mocked(logger.error);

function fireWindowErrorEvent(init: Partial<ErrorEventInit> & { error?: unknown } = {}): void {
	const event = new ErrorEvent('error', {
		message: init.message ?? '',
		filename: init.filename ?? '',
		lineno: init.lineno ?? 0,
		colno: init.colno ?? 0,
		error: init.error as Error | undefined,
	});
	window.dispatchEvent(event);
}

function fireWindowUnhandledRejection(reason: unknown): void {
	const event = new PromiseRejectionEvent('unhandledrejection', {
		// Promise.reject is caught immediately so it doesn't leak as an unhandled rejection.
		// The handler under test reads event.reason, not event.promise, so the caught
		// promise is a safe placeholder. Non-Error values are intentional here.
		// eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
		promise: Promise.reject(reason).catch(() => undefined),
		reason,
	});
	window.dispatchEvent(event);
}

describe('installGlobalErrorHandlers', () => {
	let cleanup: () => void;

	beforeEach(() => {
		vi.clearAllMocks();
		cleanup = installGlobalErrorHandlers();
	});

	afterEach(() => {
		cleanup();
		vi.clearAllMocks();
	});

	describe('window "error" handler', () => {
		it('logs a basic error event with message, filename, line, and column', () => {
			fireWindowErrorEvent({
				message: 'Something went wrong',
				filename: 'app.js',
				lineno: 10,
				colno: 5,
			});

			expect(mockLoggerError).toHaveBeenCalledOnce();
			expect(mockLoggerError).toHaveBeenCalledWith(
				'[GlobalError] Something went wrong (app.js:10:5)',
			);
		});

		it('appends the stack trace when event.error is an Error with a stack', () => {
			const err = new Error('boom');
			err.stack = 'Error: boom\n    at someFile.ts:42:7';

			fireWindowErrorEvent({
				message: 'boom',
				filename: 'someFile.ts',
				lineno: 42,
				colno: 7,
				error: err,
			});

			expect(mockLoggerError).toHaveBeenCalledOnce();
			expect(mockLoggerError).toHaveBeenCalledWith(
				`[GlobalError] boom (someFile.ts:42:7)\n${err.stack}`,
			);
		});

		it('does not append stack when event.error is an Error without a stack', () => {
			const err = new Error('no stack');
			err.stack = undefined;

			fireWindowErrorEvent({
				message: 'no stack',
				filename: 'file.ts',
				lineno: 1,
				colno: 1,
				error: err,
			});

			expect(mockLoggerError).toHaveBeenCalledWith('[GlobalError] no stack (file.ts:1:1)');
		});

		it('does not append stack when there is no error property', () => {
			fireWindowErrorEvent({ message: 'plain error', filename: 'x.ts', lineno: 3, colno: 2 });

			expect(mockLoggerError).toHaveBeenCalledWith('[GlobalError] plain error (x.ts:3:2)');
		});
	});

	describe('window "unhandledrejection" handler', () => {
		it('logs an Error reason with name and message', () => {
			const err = new Error('async failure');
			err.stack = undefined;

			fireWindowUnhandledRejection(err);

			expect(mockLoggerError).toHaveBeenCalledOnce();
			expect(mockLoggerError).toHaveBeenCalledWith(
				'[UnhandledRejection] Error: async failure',
			);
		});

		it('appends the stack trace when the Error reason has a stack', () => {
			const err = new Error('async failure');
			err.stack = 'Error: async failure\n    at async.ts:99:3';

			fireWindowUnhandledRejection(err);

			expect(mockLoggerError).toHaveBeenCalledOnce();
			expect(mockLoggerError).toHaveBeenCalledWith(
				`[UnhandledRejection] Error: async failure\n${err.stack}`,
			);
		});

		it('uses the subclass name, not a hardcoded "Error" string', () => {
			class CustomError extends Error {
				override name = 'CustomError';
			}
			const err = new CustomError('custom problem');
			err.stack = undefined;

			fireWindowUnhandledRejection(err);

			expect(mockLoggerError).toHaveBeenCalledWith(
				'[UnhandledRejection] CustomError: custom problem',
			);
		});

		it('converts a non-Error reason to a string log message', () => {
			fireWindowUnhandledRejection('something bad happened');

			expect(mockLoggerError).toHaveBeenCalledWith(
				'[UnhandledRejection] something bad happened',
			);
		});
	});
});
