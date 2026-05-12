export const SETTINGS_SCHEMA_VERSION = 1;

export interface VocylSettings {
	schemaVersion: number;
	testProperty: string;
}

export const DEFAULT_SETTINGS: VocylSettings = {
	schemaVersion: SETTINGS_SCHEMA_VERSION,
	testProperty: '',
};
