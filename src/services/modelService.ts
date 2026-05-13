import { invoke } from '@tauri-apps/api/core';

export const modelService = {
	listDownloadedFiles: (dir: 'tts' | 'stt') => invoke<string[]>('list_downloaded_files', { dir }),

	downloadFile: (dir: 'tts' | 'stt', filename: string, url: string) =>
		invoke<void>('download_file', { dir, filename, url }),

	deleteFile: (dir: 'tts' | 'stt', filename: string) =>
		invoke<void>('delete_file', { dir, filename }),

	isPiperInstalled: () => invoke<boolean>('is_piper_installed'),
	isWhisperInstalled: () => invoke<boolean>('is_whisper_installed'),
	isSherpaInstalled: () => invoke<boolean>('is_sherpa_tts_installed'),
	isKokoroInstalled: () => invoke<boolean>('is_kokoro_installed'),

	downloadPiper: () => invoke<void>('download_piper'),
	downloadWhisper: () => invoke<void>('download_whisper'),
	downloadSherpa: () => invoke<void>('download_sherpa_tts'),
	downloadKokoro: () => invoke<void>('download_kokoro_package'),
};
