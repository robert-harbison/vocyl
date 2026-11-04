import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';

import { DEFAULT_SETTINGS, SETTINGS_SCHEMA_VERSION, type SttModelSize } from '@models/settings';
import { useSettingsContext } from '@contexts/SettingsContext';
import { modelService } from '@services/modelService';
import { settingsService } from '@services/settingsService';
import { systemService } from '@services/systemService';
import { logger } from '@services/logService';
import { getRecommendedSttModel } from '@utils/modelUtils';
import type { DownloadStates } from '@screens/SettingsScreen/ModelRow';
import { STT_MODELS } from '@/constants/models';

import WizardShell from './WizardShell';
import WelcomeStep from './WelcomeStep';
import TtsStep from './TtsStep';
import SttStep from './SttStep';
import KeybindsStep from './KeybindsStep';
import FinishStep from './FinishStep';

type WizardStep = 'welcome' | 'tts' | 'stt' | 'keybinds' | 'finish';

interface DownloadProgressEvent {
	dir: string;
	filename: string;
	progress: number;
}

interface DownloadCompleteEvent {
	dir: string;
	filename: string;
}

export default function SetupScreen() {
	const { saveSettings } = useSettingsContext();

	// ── Wizard navigation ─────────────────────────────────────────────────────
	const [step, setStep] = useState<WizardStep>('welcome');
	const [isSaving, setIsSaving] = useState(false);

	// ── User selections (persist across steps) ────────────────────────────────
	const [pttShortcut, setPttShortcut] = useState(DEFAULT_SETTINGS.pttShortcut);
	const [ttsShortcut, setTtsShortcut] = useState(DEFAULT_SETTINGS.ttsShortcut);
	const [selectedSttModel, setSelectedSttModel] = useState<SttModelSize | null>(null);

	// ── Download / install state ──────────────────────────────────────────────
	const [kokoroInstalled, setKokoroInstalled] = useState(false);
	const [sttFiles, setSttFiles] = useState<string[]>([]);
	const [dlStates, setDlStates] = useState<DownloadStates>({});

	// ── System info for recommendation ────────────────────────────────────────
	const [systemRamGb, setSystemRamGb] = useState<number | null>(null);

	const recommendedSttModel = systemRamGb !== null ? getRecommendedSttModel(systemRamGb) : null;

	// ── Auto-select the first downloaded STT model (wizard only) ─────────────
	// When a model finishes downloading and nothing is selected yet, immediately
	// mark that model as active so the user doesn't have to click "Use" manually.
	useEffect(() => {
		if (selectedSttModel !== null) return; // already have a selection
		const firstDownloaded = STT_MODELS.find((m) => sttFiles.includes(m.file.filename));
		if (firstDownloaded) {
			logger.info(
				`SetupScreen: auto-selecting first downloaded model '${firstDownloaded.id}'`,
			);
			setSelectedSttModel(firstDownloaded.id);
		}
	}, [sttFiles, selectedSttModel]);

	// ── Data loading ──────────────────────────────────────────────────────────

	const refreshSttFiles = useCallback(async () => {
		const files = await modelService.listDownloadedFiles('stt');
		setSttFiles(files);
	}, []);

	const refreshBinaries = useCallback(async () => {
		const kokoro = await modelService.isKokoroInstalled();
		setKokoroInstalled(kokoro);
	}, []);

	useEffect(() => {
		void (async () => {
			logger.info('SetupScreen: loading initial state');
			const [, , sysInfo] = await Promise.all([
				refreshSttFiles(),
				refreshBinaries(),
				systemService.getSystemInfo(),
			]);
			setSystemRamGb(sysInfo.total_ram_mb / 1024);
			logger.info(
				`SetupScreen: system RAM ${String(sysInfo.total_ram_mb)} MB, CPUs ${String(sysInfo.cpu_count)}`,
			);
		})();
	}, [refreshSttFiles, refreshBinaries]);

	// ── Download event listeners ──────────────────────────────────────────────

	useEffect(() => {
		const unsubs: (() => void)[] = [];

		void (async () => {
			unsubs.push(
				await listen<DownloadProgressEvent>('download-progress', (e) => {
					const key = `${e.payload.dir}/${e.payload.filename}`;
					setDlStates((prev) => ({ ...prev, [key]: e.payload.progress }));
				}),
			);

			unsubs.push(
				await listen<DownloadCompleteEvent>('download-complete', (e) => {
					const key = `${e.payload.dir}/${e.payload.filename}`;
					setDlStates((prev) => {
						const { [key]: _removed, ...rest } = prev;
						return rest;
					});
					const dir = e.payload.dir;
					if (dir === 'stt') void refreshSttFiles();
					if (dir === 'tts') void refreshBinaries();
				}),
			);
		})();

		return () => {
			unsubs.forEach((fn) => {
				fn();
			});
		};
	}, [refreshSttFiles, refreshBinaries]);

	// ── Finish wizard ─────────────────────────────────────────────────────────

	const handleFinish = async () => {
		logger.info('SetupScreen: wizard complete - saving settings');
		setIsSaving(true);
		try {
			await saveSettings({
				...DEFAULT_SETTINGS,
				schemaVersion: SETTINGS_SCHEMA_VERSION,
				activeSttModel: selectedSttModel,
				pttShortcut,
				ttsShortcut,
			});
			await settingsService.updateShortcuts(pttShortcut, ttsShortcut);
			logger.info('SetupScreen: settings saved');
		} catch (err) {
			logger.error(`SetupScreen: save failed - ${String(err)}`);
			setIsSaving(false);
		}
	};

	// ── Render ────────────────────────────────────────────────────────────────

	return (
		<WizardShell step={step}>
			{step === 'welcome' && (
				<WelcomeStep
					onNext={() => {
						setStep('tts');
					}}
				/>
			)}

			{step === 'tts' && (
				<TtsStep
					kokoroInstalled={kokoroInstalled}
					dlStates={dlStates}
					onDownload={async () => {
						await modelService.downloadKokoro();
						await refreshBinaries();
					}}
					onDelete={async () => {
						await modelService.deleteKokoro();
						await refreshBinaries();
					}}
					onNext={() => {
						setStep('stt');
					}}
					onSkip={() => {
						setStep('stt');
					}}
				/>
			)}

			{step === 'stt' && (
				<SttStep
					sttFiles={sttFiles}
					dlStates={dlStates}
					selectedModel={selectedSttModel}
					recommendedModelId={recommendedSttModel}
					systemRamGb={systemRamGb}
					onSelectModel={setSelectedSttModel}
					onRefresh={() => void refreshSttFiles()}
					onNext={() => {
						setStep('keybinds');
					}}
					onSkip={() => {
						setStep('keybinds');
					}}
				/>
			)}

			{step === 'keybinds' && (
				<KeybindsStep
					pttShortcut={pttShortcut}
					ttsShortcut={ttsShortcut}
					onPttChange={setPttShortcut}
					onTtsChange={setTtsShortcut}
					onFinish={() => {
						setStep('finish');
					}}
					onSkip={() => {
						setStep('finish');
					}}
				/>
			)}

			{step === 'finish' && (
				<FinishStep
					pttShortcut={pttShortcut}
					ttsShortcut={ttsShortcut}
					onClose={() => void handleFinish()}
					isSaving={isSaving}
				/>
			)}
		</WizardShell>
	);
}
