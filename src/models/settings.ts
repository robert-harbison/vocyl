export const SETTINGS_SCHEMA_VERSION = 1;

export type SttModelSize = 'tiny' | 'base' | 'small' | 'medium' | 'large-v3';

export interface VocylSettings {
	schemaVersion: number;
	inputDeviceId: string | null;
	outputDeviceId: string | null;
	pttShortcut: string;
	ttsShortcut: string;
	ttsVoice: string | null;
	activeSttModel: SttModelSize | null;
}

export const DEFAULT_SETTINGS: VocylSettings = {
	schemaVersion: SETTINGS_SCHEMA_VERSION,
	inputDeviceId: null,
	outputDeviceId: null,
	pttShortcut: 'Alt+A',
	ttsShortcut: 'Alt+S',
	ttsVoice: 'af',
	activeSttModel: null,
};
