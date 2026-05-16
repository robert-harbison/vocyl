import { invoke } from '@tauri-apps/api/core';

export const modelService = {
	listDownloadedFiles: (dir: 'tts' | 'stt') => invoke<string[]>('list_downloaded_files', { dir }),

	downloadFile: (dir: 'tts' | 'stt', filename: string, url: string) =>
		invoke<void>('download_file', { dir, filename, url }),

	deleteFile: (dir: 'tts' | 'stt', filename: string) =>
		invoke<void>('delete_file', { dir, filename }),

	isKokoroInstalled: () => invoke<boolean>('is_kokoro_installed'),

	downloadKokoro: () => invoke<void>('download_kokoro_package'),
};
