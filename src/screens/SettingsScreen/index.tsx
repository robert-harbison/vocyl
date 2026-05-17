declare const __APP_VERSION__: string;

import { useState, useEffect, useCallback } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { listen } from '@tauri-apps/api/event';

import { SETTINGS_SCHEMA_VERSION, type SttModelSize } from '@models/settings';
import { useSettingsContext } from '@contexts/SettingsContext';
import Button from '@components/Button';
import Dropdown from '@components/Dropdown';
import KeybindRecorder from '@components/KeybindRecorder';
import SettingsRow from './SettingsRow';
import SettingsSection from './SettingsSection';
import { appService } from '@services/appService';
import { audioService, type AudioDevice } from '@services/audioService';
import { modelService } from '@services/modelService';
import { settingsService } from '@services/settingsService';
import { logger } from '@services/logService';
import { KOKORO_VOICES, STT_MODELS } from '@/constants/models';
import { Check } from 'lucide-react';
import ModelRow, { type DownloadStates } from './ModelRow';
import BinaryRow from './BinaryRow';
import { systemService } from '@services/systemService';
import { getRecommendedSttModel } from '@utils/modelUtils';
import logoWhiteImage from '@assets/FullLogoColor_WhiteText.png';

// ── Local types ───────────────────────────────────────────────────────────────

interface FormValues {
	inputDeviceId: string | null;
	outputDeviceId: string | null;
	pttShortcut: string;
	ttsShortcut: string;
	ttsVoice: string | null;
	activeSttModel: SttModelSize | null;
}

interface DownloadProgressEvent {
	dir: string;
	filename: string;
	progress: number;
}

interface DownloadCompleteEvent {
	dir: string;
	filename: string;
}

// ── Main screen ───────────────────────────────────────────────────────────────

export default function SettingsScreen() {
	const { settings, saveSettings } = useSettingsContext();
	const [saved, setSaved] = useState(false);

	const [inputDevices, setInputDevices] = useState<AudioDevice[]>([]);
	const [outputDevices, setOutputDevices] = useState<AudioDevice[]>([]);
	const [sttFiles, setSttFiles] = useState<string[]>([]);
	const [kokoroInstalled, setKokoroInstalled] = useState(false);
	const [dlStates, setDlStates] = useState<DownloadStates>({});
	const [systemRamGb, setSystemRamGb] = useState<number | null>(null);

	const {
		control,
		handleSubmit,
		watch,
		setValue,
		trigger,
		formState: { isSubmitting, errors },
	} = useForm<FormValues>({
		defaultValues: {
			inputDeviceId: settings.inputDeviceId ?? '',
			outputDeviceId: settings.outputDeviceId ?? '',
			pttShortcut: settings.pttShortcut,
			ttsShortcut: settings.ttsShortcut,
			ttsVoice: settings.ttsVoice,
			activeSttModel: settings.activeSttModel,
		},
	});

	const activeSttModel = watch('activeSttModel');

	// ── Data loading ──────────────────────────────────────────────────────────

	const refreshSttFiles = useCallback(async () => {
		logger.debug('SettingsScreen: refreshing STT model files');
		const files = await modelService.listDownloadedFiles('stt');
		setSttFiles(files);
	}, []);

	const refreshBinaries = useCallback(async () => {
		logger.debug('SettingsScreen: checking installed binaries');
		const kokoro = await modelService.isKokoroInstalled();
		logger.info(`SettingsScreen: kokoro=${String(kokoro)}`);
		setKokoroInstalled(kokoro);
	}, []);

	useEffect(() => {
		void (async () => {
			logger.info('SettingsScreen: loading devices and model state');
			const [inputs, outputs] = await Promise.all([
				audioService.listInputDevices(),
				audioService.listOutputDevices(),
			]);
			logger.info(
				`SettingsScreen: ${String(inputs.length)} input device(s), ${String(outputs.length)} output device(s)`,
			);
			setInputDevices(inputs);
			setOutputDevices(outputs);

			const [, , sysInfo] = await Promise.all([
				refreshSttFiles(),
				refreshBinaries(),
				systemService.getSystemInfo(),
			]);
			setSystemRamGb(sysInfo.total_ram_mb / 1024);
			logger.info('SettingsScreen: initial load complete');
		})();
	}, [refreshSttFiles, refreshBinaries]);

	// ── Download event listeners ──────────────────────────────────────────────

	useEffect(() => {
		const unsubs: (() => void)[] = [];

		void (async () => {
			unsubs.push(
				await listen<DownloadProgressEvent>('download-progress', (e) => {
					const key = `${e.payload.dir}/${e.payload.filename}`;
					logger.debug(`Download progress: ${key} ${String(e.payload.progress)}%`);
					setDlStates((prev) => ({ ...prev, [key]: e.payload.progress }));
				}),
			);

			unsubs.push(
				await listen<DownloadCompleteEvent>('download-complete', (e) => {
					const key = `${e.payload.dir}/${e.payload.filename}`;
					logger.info(`Download complete: ${key}`);
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

	// ── Submit ────────────────────────────────────────────────────────────────

	const onSubmit = async (data: FormValues) => {
		logger.info('SettingsScreen: saving settings form');
		await saveSettings({
			schemaVersion: SETTINGS_SCHEMA_VERSION,
			inputDeviceId: data.inputDeviceId === '' ? null : data.inputDeviceId,
			outputDeviceId: data.outputDeviceId === '' ? null : data.outputDeviceId,
			pttShortcut: data.pttShortcut,
			ttsShortcut: data.ttsShortcut,
			ttsVoice: data.ttsVoice,
			activeSttModel: data.activeSttModel,
		});
		await settingsService.updateShortcuts(data.pttShortcut, data.ttsShortcut);
		logger.info(
			`SettingsScreen: settings saved - PTT='${data.pttShortcut}' TTS='${data.ttsShortcut}'`,
		);
		setSaved(true);
		setTimeout(() => {
			setSaved(false);
		}, 3000);
	};

	// ── Device dropdown options ───────────────────────────────────────────────

	const inputOptions = inputDevices.map((d) => ({ value: d.id, label: d.name }));
	const outputOptions = outputDevices.map((d) => ({ value: d.id, label: d.name }));

	const recommendedSttModel = systemRamGb !== null ? getRecommendedSttModel(systemRamGb) : null;

	return (
		<div className="min-h-screen bg-bg-base text-text">
			{/* Header */}
			<header className="border-b border-border px-6 py-4 flex items-center justify-between">
				<div className="flex items-center gap-3">
					<img src={logoWhiteImage} alt="Vocyl Logo" className="h-8" />
				</div>
				<span className="text-xs text-text-faint font-mono bg-bg-elevated px-2 py-1 rounded">
					v{__APP_VERSION__}
				</span>
			</header>

			{/* Content */}
			<main className="max-w-2xl mx-auto px-6 py-8">
				<form onSubmit={(e) => void handleSubmit(onSubmit)(e)} className="space-y-8">
					{/* ── Audio ── */}
					<SettingsSection title="Audio">
						<SettingsRow
							label="Input Device"
							description="Microphone used for push-to-talk."
						>
							<Controller
								name="inputDeviceId"
								control={control}
								render={({ field }) => (
									<Dropdown
										options={inputOptions}
										value={field.value}
										onChange={field.onChange}
										placeholder="Default (System)"
										className="w-56"
									/>
								)}
							/>
						</SettingsRow>
						<SettingsRow
							label="Output Device"
							description="Speaker / headphones used for text-to-speech."
						>
							<Controller
								name="outputDeviceId"
								control={control}
								render={({ field }) => (
									<Dropdown
										options={outputOptions}
										value={field.value}
										onChange={field.onChange}
										placeholder="Default (System)"
										className="w-56"
									/>
								)}
							/>
						</SettingsRow>
					</SettingsSection>

					{/* ── Shortcuts ── */}
					<SettingsSection title="Shortcuts">
						<SettingsRow
							label="Push-to-Talk"
							description="Hold to record. Text is typed at your cursor on release."
							error={errors.pttShortcut?.message}
						>
							<Controller
								name="pttShortcut"
								control={control}
								rules={{
									validate: (val) =>
										val !== watch('ttsShortcut') ||
										'Cannot be the same as Read Selection shortcut',
								}}
								render={({ field }) => (
									<KeybindRecorder
										value={field.value}
										onChange={(v) => {
											field.onChange(v);
											void trigger('ttsShortcut');
										}}
									/>
								)}
							/>
						</SettingsRow>
						<SettingsRow
							label="Read Selection"
							description="Press once to start reading selected text. Press again to stop."
							error={errors.ttsShortcut?.message}
						>
							<Controller
								name="ttsShortcut"
								control={control}
								rules={{
									validate: (val) =>
										val !== watch('pttShortcut') ||
										'Cannot be the same as Push-to-Talk shortcut',
								}}
								render={({ field }) => (
									<KeybindRecorder
										value={field.value}
										onChange={(v) => {
											field.onChange(v);
											void trigger('pttShortcut');
										}}
									/>
								)}
							/>
						</SettingsRow>
					</SettingsSection>

					{/* ── Speech-to-Text ── */}
					<SettingsSection title="Speech-to-Text">
						<div className="px-4 py-2">
							<p className="text-xs font-semibold uppercase tracking-widest text-text-faint mb-2">
								Whisper Models
							</p>
							<div className="bg-bg-base rounded-lg border border-border divide-y divide-border">
								{STT_MODELS.map((model) => (
									<ModelRow
										key={model.id}
										id={model.id}
										label={model.label}
										sizeMb={model.sizeMb}
										files={[model.file]}
										dir="stt"
										downloadedFiles={sttFiles}
										dlStates={dlStates}
										activeId={activeSttModel}
										recommended={recommendedSttModel === model.id}
										onUse={(id) => {
											setValue('activeSttModel', id as SttModelSize);
										}}
										onRefresh={() => void refreshSttFiles()}
									/>
								))}
							</div>
						</div>
					</SettingsSection>

					{/* ── Text-to-Speech ── */}
					<SettingsSection title="Text-to-Speech">
						<BinaryRow
							label="Kokoro English Model"
							description="Voice model + phoneme data (~90 MB). The inference runtime is built into the app."
							isInstalled={kokoroInstalled}
							dlKey="tts/kokoro"
							dlStates={dlStates}
							onDownload={async () => {
								await modelService.downloadKokoro();
								await refreshBinaries();
							}}
							onDelete={async () => {
								await modelService.deleteKokoro();
								await refreshBinaries();
							}}
						/>

						<SettingsRow
							label="Voice"
							description="Speaker voice used for Kokoro synthesis."
						>
							<Controller
								name="ttsVoice"
								control={control}
								render={({ field }) => (
									<Dropdown
										options={KOKORO_VOICES}
										value={field.value}
										onChange={field.onChange}
										placeholder="Select voice..."
										className="w-56"
									/>
								)}
							/>
						</SettingsRow>
					</SettingsSection>

					{/* ── Footer ── */}
					<div className="flex items-center justify-between gap-3">
						<Button
							type="button"
							variant="destructive"
							onClick={() => void appService.exit()}
						>
							Exit Vocyl
						</Button>

						<div className="flex items-center gap-3">
							{saved && (
								<span className="text-sm text-success flex items-center gap-1.5">
									<Check className="w-4 h-4" />
									Saved
								</span>
							)}
							<Button type="submit" loading={isSubmitting}>
								Save Settings
							</Button>
						</div>
					</div>
				</form>
			</main>
		</div>
	);
}
