import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import { DEFAULT_SETTINGS, type VocylSettings } from '@models/settings';
import { settingsService } from '@services/settingsService';
import { logger } from '@services/logService';

interface SettingsContextValue {
	settings: VocylSettings;
	saveSettings: (next: VocylSettings) => Promise<void>;
	loading: boolean;
	exists: boolean | null;
	setExists: (exists: boolean) => void;
	error: string | null;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
	const [settings, setSettings] = useState<VocylSettings>(DEFAULT_SETTINGS);
	const [loading, setLoading] = useState(true);
	const [exists, setExists] = useState<boolean | null>(null);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		async function init() {
			logger.info('SettingsContext: initialising');
			try {
				const hasSettings = await settingsService.exists();
				logger.info(`SettingsContext: settings exist = ${String(hasSettings)}`);
				setExists(hasSettings);
				if (hasSettings) {
					const loaded = await settingsService.load();
					logger.info('SettingsContext: settings loaded successfully');
					setSettings(loaded);
				}
			} catch (err) {
				const msg = err instanceof Error ? err.message : String(err);
				logger.error(`SettingsContext: init error — ${msg}`);
				setError(msg);
			} finally {
				setLoading(false);
			}
		}
		void init();
	}, []);

	const saveSettings = async (next: VocylSettings): Promise<void> => {
		logger.info('SettingsContext: saving settings');
		try {
			await settingsService.save(next);
			setSettings(next);
			setExists(true);
			logger.info('SettingsContext: settings saved successfully');
		} catch (err) {
			const msg = err instanceof Error ? err.message : String(err);
			logger.error(`SettingsContext: save error — ${msg}`);
			throw err;
		}
	};

	return (
		<SettingsContext.Provider
			value={{ settings, saveSettings, loading, exists, setExists, error }}
		>
			{children}
		</SettingsContext.Provider>
	);
}

// eslint-disable-next-line react-refresh/only-export-components -- context + hook pair is intentional
export function useSettingsContext(): SettingsContextValue {
	const ctx = useContext(SettingsContext);
	if (!ctx) {
		throw new Error('useSettingsContext must be used within a SettingsProvider');
	}
	return ctx;
}
