import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { appService } from './appService';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
	vi.clearAllMocks();
});

describe('appService', () => {
	describe('exit', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await appService.exit();
			expect(mockInvoke).toHaveBeenCalledWith('exit_app');
		});

		it('calls the command exactly once', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await appService.exit();
			expect(mockInvoke).toHaveBeenCalledOnce();
		});
	});
});
