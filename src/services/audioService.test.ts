import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { audioService } from './audioService';
import type { AudioDevice } from './audioService';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

const mockInputDevices: AudioDevice[] = [
	{ id: 'input-1', name: 'Microphone (USB)' },
	{ id: 'input-2', name: 'Line In' },
];

const mockOutputDevices: AudioDevice[] = [
	{ id: 'output-1', name: 'Speakers (Realtek)' },
	{ id: 'output-2', name: 'Headphones' },
];

beforeEach(() => {
	vi.clearAllMocks();
});

describe('audioService', () => {
	describe('listInputDevices', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(mockInputDevices);
			await audioService.listInputDevices();
			expect(mockInvoke).toHaveBeenCalledWith('list_input_devices');
		});

		it('returns the list of input devices', async () => {
			mockInvoke.mockResolvedValueOnce(mockInputDevices);
			expect(await audioService.listInputDevices()).toEqual(mockInputDevices);
		});

		it('returns an empty array when no input devices are available', async () => {
			mockInvoke.mockResolvedValueOnce([]);
			expect(await audioService.listInputDevices()).toEqual([]);
		});
	});

	describe('listOutputDevices', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(mockOutputDevices);
			await audioService.listOutputDevices();
			expect(mockInvoke).toHaveBeenCalledWith('list_output_devices');
		});

		it('returns the list of output devices', async () => {
			mockInvoke.mockResolvedValueOnce(mockOutputDevices);
			expect(await audioService.listOutputDevices()).toEqual(mockOutputDevices);
		});

		it('returns an empty array when no output devices are available', async () => {
			mockInvoke.mockResolvedValueOnce([]);
			expect(await audioService.listOutputDevices()).toEqual([]);
		});
	});
});
