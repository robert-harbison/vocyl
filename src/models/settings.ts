export const SETTINGS_SCHEMA_VERSION = 2;

export type TtsEngine = 'kokoro' | 'piper';
export type SttModelSize = 'tiny' | 'base' | 'small' | 'medium' | 'large-v3';

export interface VocylSettings {
	schemaVersion: number;
	inputDeviceId: string | null;
	outputDeviceId: string | null;
	pttShortcut: string;
	ttsShortcut: string;
	ttsEngine: TtsEngine;
	activeTtsModelId: string | null;
	ttsVoice: string | null;
	activeSttModel: SttModelSize | null;
}

export const DEFAULT_SETTINGS: VocylSettings = {
	schemaVersion: SETTINGS_SCHEMA_VERSION,
	inputDeviceId: null,
	outputDeviceId: null,
	pttShortcut: 'CmdOrCtrl+Shift+F8',
	ttsShortcut: 'CmdOrCtrl+Shift+F9',
	ttsEngine: 'kokoro',
	activeTtsModelId: null,
	ttsVoice: 'af_heart',
	activeSttModel: null,
};
