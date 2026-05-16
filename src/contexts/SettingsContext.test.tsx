import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import React from 'react';
import { SettingsProvider, useSettingsContext } from './SettingsContext';
import * as settingsServiceModule from '@services/settingsService';
import { SETTINGS_SCHEMA_VERSION, DEFAULT_SETTINGS } from '@models/settings';
import type { VocylSettings } from '@models/settings';

vi.mock('@services/logService');
vi.mock('@services/settingsService', () => ({
	settingsService: {
		exists: vi.fn(),
		load: vi.fn(),
		save: vi.fn(),
	},
}));

const mockService = vi.mocked(settingsServiceModule.settingsService);

const mockSettings: VocylSettings = {
	schemaVersion: SETTINGS_SCHEMA_VERSION,
	inputDeviceId: null,
	outputDeviceId: null,
	pttShortcut: 'Alt+A',
	ttsShortcut: 'Alt+S',
	ttsVoice: 'af_heart',
	activeSttModel: null,
};

const wrapper = ({ children }: { children: React.ReactNode }) =>
	React.createElement(SettingsProvider, null, children);

beforeEach(() => {
	vi.clearAllMocks();
});

describe('SettingsContext', () => {
	describe('initial load - settings exist', () => {
		it('loads and exposes settings', async () => {
			mockService.exists.mockResolvedValueOnce(true);
			mockService.load.mockResolvedValueOnce(mockSettings);

			const { result } = renderHook(() => useSettingsContext(), { wrapper });

			expect(result.current.loading).toBe(true);

			await waitFor(() => {
				expect(result.current.loading).toBe(false);
			});

			expect(result.current.exists).toBe(true);
			expect(result.current.settings).toEqual(mockSettings);
		});
	});

	describe('initial load - no settings yet', () => {
		it('sets exists to false and keeps default settings', async () => {
			mockService.exists.mockResolvedValueOnce(false);

			const { result } = renderHook(() => useSettingsContext(), { wrapper });

			await waitFor(() => {
				expect(result.current.loading).toBe(false);
			});

			expect(result.current.exists).toBe(false);
			expect(result.current.settings).toEqual(DEFAULT_SETTINGS);
		});
	});

	describe('initial load - backend error', () => {
		it('sets error and stops loading', async () => {
			mockService.exists.mockRejectedValueOnce(new Error('backend offline'));

			const { result } = renderHook(() => useSettingsContext(), { wrapper });

			await waitFor(() => {
				expect(result.current.loading).toBe(false);
			});

			expect(result.current.error).toBe('backend offline');
		});
	});

	describe('saveSettings', () => {
		it('calls the service and updates state', async () => {
			mockService.exists.mockResolvedValueOnce(false);
			mockService.save.mockResolvedValueOnce(undefined);

			const { result } = renderHook(() => useSettingsContext(), { wrapper });
			await waitFor(() => {
				expect(result.current.loading).toBe(false);
			});

			await act(async () => {
				await result.current.saveSettings(mockSettings);
			});

			expect(mockService.save).toHaveBeenCalledWith(mockSettings);
			expect(result.current.settings).toEqual(mockSettings);
			expect(result.current.exists).toBe(true);
		});
	});

	describe('useSettingsContext outside provider', () => {
		it('throws a helpful error', () => {
			expect(() => renderHook(() => useSettingsContext())).toThrow(
				'useSettingsContext must be used within a SettingsProvider',
			);
		});
	});
});
