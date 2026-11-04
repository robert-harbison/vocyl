import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { modelService } from './modelService';

vi.mock('@tauri-apps/api/core');

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
	vi.clearAllMocks();
});

describe('modelService', () => {
	describe('listDownloadedFiles', () => {
		it('calls the correct Tauri command with dir=tts', async () => {
			mockInvoke.mockResolvedValueOnce(['kokoro.onnx']);
			await modelService.listDownloadedFiles('tts');
			expect(mockInvoke).toHaveBeenCalledWith('list_downloaded_files', { dir: 'tts' });
		});

		it('calls the correct Tauri command with dir=stt', async () => {
			mockInvoke.mockResolvedValueOnce(['tiny.bin']);
			await modelService.listDownloadedFiles('stt');
			expect(mockInvoke).toHaveBeenCalledWith('list_downloaded_files', { dir: 'stt' });
		});

		it('returns the list of downloaded filenames', async () => {
			mockInvoke.mockResolvedValueOnce(['kokoro.onnx', 'voices.bin']);
			expect(await modelService.listDownloadedFiles('tts')).toEqual([
				'kokoro.onnx',
				'voices.bin',
			]);
		});

		it('returns an empty array when no files are downloaded', async () => {
			mockInvoke.mockResolvedValueOnce([]);
			expect(await modelService.listDownloadedFiles('stt')).toEqual([]);
		});
	});

	describe('downloadFile', () => {
		it('calls the correct Tauri command with the expected arguments', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.downloadFile(
				'tts',
				'kokoro.onnx',
				'https://example.com/kokoro.onnx',
			);
			expect(mockInvoke).toHaveBeenCalledWith('download_file', {
				dir: 'tts',
				filename: 'kokoro.onnx',
				url: 'https://example.com/kokoro.onnx',
			});
		});

		it('works for the stt dir', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.downloadFile('stt', 'tiny.bin', 'https://example.com/tiny.bin');
			expect(mockInvoke).toHaveBeenCalledWith('download_file', {
				dir: 'stt',
				filename: 'tiny.bin',
				url: 'https://example.com/tiny.bin',
			});
		});
	});

	describe('deleteFile', () => {
		it('calls the correct Tauri command with the expected arguments', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.deleteFile('tts', 'kokoro.onnx');
			expect(mockInvoke).toHaveBeenCalledWith('delete_file', {
				dir: 'tts',
				filename: 'kokoro.onnx',
			});
		});

		it('works for the stt dir', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.deleteFile('stt', 'tiny.bin');
			expect(mockInvoke).toHaveBeenCalledWith('delete_file', {
				dir: 'stt',
				filename: 'tiny.bin',
			});
		});
	});

	describe('isKokoroInstalled', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(true);
			await modelService.isKokoroInstalled();
			expect(mockInvoke).toHaveBeenCalledWith('is_kokoro_installed');
		});

		it('returns true when Kokoro is installed', async () => {
			mockInvoke.mockResolvedValueOnce(true);
			expect(await modelService.isKokoroInstalled()).toBe(true);
		});

		it('returns false when Kokoro is not installed', async () => {
			mockInvoke.mockResolvedValueOnce(false);
			expect(await modelService.isKokoroInstalled()).toBe(false);
		});
	});

	describe('downloadKokoro', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.downloadKokoro();
			expect(mockInvoke).toHaveBeenCalledWith('download_kokoro_package');
		});
	});

	describe('deleteKokoro', () => {
		it('calls the correct Tauri command', async () => {
			mockInvoke.mockResolvedValueOnce(undefined);
			await modelService.deleteKokoro();
			expect(mockInvoke).toHaveBeenCalledWith('delete_kokoro_package');
		});
	});
});
