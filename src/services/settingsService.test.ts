import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { settingsService } from './settingsService';
import type { VocylSettings } from '@models/settings';
import { SETTINGS_SCHEMA_VERSION } from '@models/settings';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

const mockSettings: VocylSettings = {
	schemaVersion: SETTINGS_SCHEMA_VERSION,
	inputDeviceId: null,
	outputDeviceId: null,
	pttShortcut: 'CmdOrCtrl+Shift+F8',
	ttsShortcut: 'CmdOrCtrl+Shift+F9',
	ttsVoice: 'af_heart',
	activeSttModel: null,
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('settingsService', () => {
	describe('exists', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(true);
			await settingsService.exists();
			expect(mockInvoke).toHaveBeenCalledWith('settings_exist');
		});

		it('returns true when settings exist', async () => {
			mockInvoke.mockResolvedValueOnce(true);
			expect(await settingsService.exists()).toBe(true);
		});

		it('returns false when settings do not exist', async () => {
			mockInvoke.mockResolvedValueOnce(false);
			expect(await settingsService.exists()).toBe(false);
		});
	});

	describe('load', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(mockSettings);
			await settingsService.load();
			expect(mockInvoke).toHaveBeenCalledWith('load_settings');
		});

		it('returns the loaded settings', async () => {
			mockInvoke.mockResolvedValueOnce(mockSettings);
			expect(await settingsService.load()).toEqual(mockSettings);
		});
	});

	describe('save', () => {
		it('calls the correct Tauri command with the settings payload', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await settingsService.save(mockSettings);
			expect(mockInvoke).toHaveBeenCalledWith('save_settings', {
				settings: mockSettings,
			});
		});
	});
});
