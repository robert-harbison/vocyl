import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('./logService');

import { logger } from './logService';

beforeEach(() => {
	vi.clearAllMocks();
});

describe('logger', () => {
	it('exposes info, warn, error, debug and trace methods', () => {
		expect(typeof logger.info).toBe('function');
		expect(typeof logger.warn).toBe('function');
		expect(typeof logger.error).toBe('function');
		expect(typeof logger.debug).toBe('function');
		expect(typeof logger.trace).toBe('function');
	});

	it('records calls to info', () => {
		logger.info('hello');
		expect(logger.info).toHaveBeenCalledWith('hello');
		expect(logger.info).toHaveBeenCalledTimes(1);
	});

	it('records calls to warn', () => {
		logger.warn('something might be wrong');
		expect(logger.warn).toHaveBeenCalledWith('something might be wrong');
		expect(logger.warn).toHaveBeenCalledTimes(1);
	});

	it('records calls to error', () => {
		logger.error('something went wrong');
		expect(logger.error).toHaveBeenCalledWith('something went wrong');
		expect(logger.error).toHaveBeenCalledTimes(1);
	});

	it('records calls to debug', () => {
		logger.debug('debug message');
		expect(logger.debug).toHaveBeenCalledWith('debug message');
		expect(logger.debug).toHaveBeenCalledTimes(1);
	});

	it('records calls to trace', () => {
		logger.trace('trace message');
		expect(logger.trace).toHaveBeenCalledWith('trace message');
		expect(logger.trace).toHaveBeenCalledTimes(1);
	});

	it('tracks each method independently', () => {
		logger.info('a');
		logger.warn('b');
		expect(logger.info).toHaveBeenCalledTimes(1);
		expect(logger.warn).toHaveBeenCalledTimes(1);
		expect(logger.error).not.toHaveBeenCalled();
		expect(logger.debug).not.toHaveBeenCalled();
		expect(logger.trace).not.toHaveBeenCalled();
	});
});
