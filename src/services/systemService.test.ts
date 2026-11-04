import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { systemService } from './systemService';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
	vi.clearAllMocks();
});

describe('systemService', () => {
	describe('getSystemInfo', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce({ total_ram_mb: 16384, cpu_count: 8 });
			await systemService.getSystemInfo();
			expect(mockInvoke).toHaveBeenCalledWith('get_system_info');
		});

		it('returns the system info payload', async () => {
			const payload = { total_ram_mb: 8192, cpu_count: 4 };
			mockInvoke.mockResolvedValueOnce(payload);
			const result = await systemService.getSystemInfo();
			expect(result).toEqual(payload);
		});

		it('calls the command exactly once', async () => {
			mockInvoke.mockResolvedValueOnce({ total_ram_mb: 4096, cpu_count: 2 });
			await systemService.getSystemInfo();
			expect(mockInvoke).toHaveBeenCalledOnce();
		});
	});
});
