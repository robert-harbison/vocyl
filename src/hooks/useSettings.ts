import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import type { VocylSettings } from '../types/settings';
import { DEFAULT_SETTINGS } from '../types/settings';

export function useSettings() {
	const [settings, setSettings] = useState<VocylSettings>(DEFAULT_SETTINGS);
	const [loading, setLoading] = useState(true);
	const [exists, setExists] = useState<boolean | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		async function init() {
			try {
				const hasSettings = await invoke<boolean>('settings_exist');
				setExists(hasSettings);
				if (hasSettings) {
					const loaded = await invoke<VocylSettings>('load_settings');
					setSettings(loaded);
				}
			} catch (err) {
				setError(String(err));
			} finally {
				setLoading(false);
			}
		}
		void init();
	}, []);

	const saveSettings = async (next: VocylSettings): Promise<void> => {
		await invoke('save_settings', { settings: next });
		setSettings(next);
		setExists(true);
	};

	return {
		settings,
		setSettings,
		saveSettings,
		loading,
		exists,
		setExists,
		error,
	};
}
