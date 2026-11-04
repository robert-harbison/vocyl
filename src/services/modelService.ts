import { invoke } from '@tauri-apps/api/core';

export const modelService = {
	listDownloadedFiles: (dir: 'tts' | 'stt') => invoke<string[]>('list_downloaded_files', { dir }),

	downloadFile: (dir: 'tts' | 'stt', filename: string, url: string) =>
		invoke('download_file', { dir, filename, url }),

	deleteFile: (dir: 'tts' | 'stt', filename: string) => invoke('delete_file', { dir, filename }),

	isKokoroInstalled: () => invoke<boolean>('is_kokoro_installed'),

	downloadKokoro: () => invoke('download_kokoro_package'),

	deleteKokoro: () => invoke('delete_kokoro_package'),
};
