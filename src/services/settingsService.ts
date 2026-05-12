import { invoke } from '@tauri-apps/api/core';
import type { VocylSettings } from '@models/settings';

export const settingsService = {
	exists: () => invoke<boolean>('settings_exist'),
	load: () => invoke<VocylSettings>('load_settings'),
	save: (settings: VocylSettings) => invoke('save_settings', { settings }),
};
