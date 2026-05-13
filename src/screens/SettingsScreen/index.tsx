declare const __APP_VERSION__: string;

import { useState, useEffect, useCallback, type JSX } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { listen } from '@tauri-apps/api/event';
import { clsx } from 'clsx';

import { SETTINGS_SCHEMA_VERSION, type TtsEngine, type SttModelSize } from '@models/settings';
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
import { KOKORO_VOICES, PIPER_VOICES, STT_MODELS, type ModelFile } from '@/constants/models';

// ── Local types ───────────────────────────────────────────────────────────────

interface FormValues {
	inputDeviceId: string | null;
	outputDeviceId: string | null;
	pttShortcut: string;
	ttsShortcut: string;
	ttsEngine: TtsEngine;
	activeTtsModelId: string | null;
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

// Key = `${dir}/${filename}`, value = 0-100 progress or 'done'
type DownloadStates = Record<string, number | 'done'>;

// ── Model download row ────────────────────────────────────────────────────────

interface ModelRowProps {
	id: string;
	label: string;
	sizeMb: number;
	files: ModelFile[];
	dir: 'tts' | 'stt';
	downloadedFiles: string[];
	dlStates: DownloadStates;
	activeId: string | null;
	onUse: (id: string) => void;
	onRefresh: (dir: 'tts' | 'stt') => void;
}

function ModelRow({
	id,
	label,
	sizeMb,
	files,
	dir,
	downloadedFiles,
	dlStates,
	activeId,
	onUse,
	onRefresh,
}: ModelRowProps): JSX.Element {
	const isDownloaded = files.every((f) => downloadedFiles.includes(f.filename));
	const isActive = activeId === id;

	const activeFile = files.find((f) => {
		const key = `${dir}/${f.filename}`;
		return typeof dlStates[key] === 'number';
	});
	const isDownloading = activeFile !== undefined;
	const progress = activeFile ? (dlStates[`${dir}/${activeFile.filename}`] as number) : 0;

	const handleDownload = useCallback(async () => {
		for (const file of files) {
			await modelService.downloadFile(dir, file.filename, file.url);
		}
		onRefresh(dir);
	}, [files, dir, onRefresh]);

	const handleDelete = useCallback(async () => {
		for (const file of files) {
			if (downloadedFiles.includes(file.filename)) {
				await modelService.deleteFile(dir, file.filename);
			}
		}
		onRefresh(dir);
	}, [files, dir, downloadedFiles, onRefresh]);

	return (
		<div className="flex items-center gap-3 px-4 py-2.5">
			<span
				className={clsx(
					'w-2 h-2 shrink-0 rounded-full',
					isDownloaded ? 'bg-success' : 'bg-text-faint/30',
				)}
			/>
			<span className="flex-1 text-sm text-text truncate">{label}</span>
			<span className="text-xs text-text-faint tabular-nums shrink-0">
				{sizeMb >= 1000 ? `${(sizeMb / 1000).toFixed(1)} GB` : `${sizeMb} MB`}
			</span>

			{isDownloading ?
				<div className="flex items-center gap-2 min-w-[112px]">
					<div className="flex-1 h-1 bg-bg-elevated rounded-full overflow-hidden">
						<div
							className="h-full bg-primary rounded-full transition-all duration-300"
							style={{ width: `${progress}%` }}
						/>
					</div>
					<span className="text-xs text-text-faint w-8 text-right">{progress}%</span>
				</div>
			: isDownloaded ?
				<div className="flex items-center gap-2">
					<button
						type="button"
						onClick={() => onUse(id)}
						className={clsx(
							'text-xs px-2 py-0.5 rounded transition-colors',
							isActive ?
								'bg-primary/10 text-primary font-medium'
							:	'text-text-faint hover:text-text',
						)}
					>
						{isActive ? '✓ Using' : 'Use'}
					</button>
					<button
						type="button"
						onClick={() => void handleDelete()}
						className="text-xs text-error hover:text-error/80 transition-colors px-1"
					>
						Delete
					</button>
				</div>
			:	<button
					type="button"
					onClick={() => void handleDownload()}
					className="text-xs text-primary hover:text-primary/80 transition-colors px-2 py-0.5 rounded border border-primary/30 hover:border-primary/60"
				>
					Download
				</button>
			}
		</div>
	);
}

// ── Binary / runtime row ──────────────────────────────────────────────────────

interface BinaryRowProps {
	label: string;
	description: string;
	isInstalled: boolean;
	dlKey: string;
	dlStates: DownloadStates;
	onDownload: () => void;
}

function BinaryRow({
	label,
	description,
	isInstalled,
	dlKey,
	dlStates,
	onDownload,
}: BinaryRowProps): JSX.Element {
	const downloading = dlStates[dlKey];
	const isDownloading = typeof downloading === 'number';
	const progress = isDownloading ? (downloading as number) : 0;

	return (
		<div className="flex items-center gap-3 px-4 py-3">
			<div className="flex-1 min-w-0">
				<p className="text-sm font-medium text-text-strong">{label}</p>
				<p className="text-xs text-text-faint mt-0.5">{description}</p>
			</div>
			{isInstalled ?
				<span className="text-xs text-success font-medium">✓ Installed</span>
			: isDownloading ?
				<div className="flex items-center gap-2 min-w-[112px]">
					<div className="flex-1 h-1 bg-bg-elevated rounded-full overflow-hidden">
						<div
							className="h-full bg-primary rounded-full transition-all duration-300"
							style={{ width: `${progress}%` }}
						/>
					</div>
					<span className="text-xs text-text-faint w-8 text-right">{progress}%</span>
				</div>
			:	<button
					type="button"
					onClick={onDownload}
					className="text-xs text-primary hover:text-primary/80 transition-colors px-2 py-0.5 rounded border border-primary/30 hover:border-primary/60"
				>
					Download
				</button>
			}
		</div>
	);
}

// ── Main screen ───────────────────────────────────────────────────────────────

export default function SettingsScreen() {
	const { settings, saveSettings } = useSettingsContext();
	const [saved, setSaved] = useState(false);

	const [inputDevices, setInputDevices] = useState<AudioDevice[]>([]);
	const [outputDevices, setOutputDevices] = useState<AudioDevice[]>([]);
	const [ttsFiles, setTtsFiles] = useState<string[]>([]);
	const [sttFiles, setSttFiles] = useState<string[]>([]);
	const [piperInstalled, setPiperInstalled] = useState(false);
	const [whisperInstalled, setWhisperInstalled] = useState(false);
	const [kokoroInstalled, setKokoroInstalled] = useState(false);
	const [dlStates, setDlStates] = useState<DownloadStates>({});

	const {
		control,
		handleSubmit,
		watch,
		setValue,
		formState: { isSubmitting },
	} = useForm<FormValues>({
		defaultValues: {
			inputDeviceId: settings.inputDeviceId,
			outputDeviceId: settings.outputDeviceId,
			pttShortcut: settings.pttShortcut,
			ttsShortcut: settings.ttsShortcut,
			ttsEngine: settings.ttsEngine,
			activeTtsModelId: settings.activeTtsModelId,
			ttsVoice: settings.ttsVoice,
			activeSttModel: settings.activeSttModel,
		},
	});

	const ttsEngine = watch('ttsEngine');
	const activeTtsModelId = watch('activeTtsModelId');
	const activeSttModel = watch('activeSttModel');

	// ── Data loading ──────────────────────────────────────────────────────────

	const refreshFiles = useCallback(async (dir?: 'tts' | 'stt' | 'both') => {
		if (!dir || dir === 'tts' || dir === 'both') {
			const files = await modelService.listDownloadedFiles('tts');
			setTtsFiles(files);
		}
		if (!dir || dir === 'stt' || dir === 'both') {
			const files = await modelService.listDownloadedFiles('stt');
			setSttFiles(files);
		}
	}, []);

	const refreshBinaries = useCallback(async () => {
		const [piper, whisper, kokoro] = await Promise.all([
			modelService.isPiperInstalled(),
			modelService.isWhisperInstalled(),
			modelService.isKokoroInstalled(),
		]);
		setPiperInstalled(piper);
		setWhisperInstalled(whisper);
		setKokoroInstalled(kokoro);
	}, []);

	useEffect(() => {
		void (async () => {
			const [inputs, outputs] = await Promise.all([
				audioService.listInputDevices(),
				audioService.listOutputDevices(),
			]);
			setInputDevices(inputs);
			setOutputDevices(outputs);

			await Promise.all([refreshFiles('both'), refreshBinaries()]);
		})();
	}, [refreshFiles, refreshBinaries]);

	// ── Download event listeners ──────────────────────────────────────────────

	useEffect(() => {
		const unsubs: Array<() => void> = [];

		listen<DownloadProgressEvent>('download-progress', (e) => {
			const key = `${e.payload.dir}/${e.payload.filename}`;
			setDlStates((prev) => ({ ...prev, [key]: e.payload.progress }));
		}).then((fn) => unsubs.push(fn));

		listen<DownloadCompleteEvent>('download-complete', (e) => {
			const key = `${e.payload.dir}/${e.payload.filename}`;
			setDlStates((prev) => {
				const next = { ...prev };
				delete next[key];
				return next;
			});
			const dir = e.payload.dir as 'tts' | 'stt' | string;
			if (dir === 'tts' || dir === 'stt') void refreshFiles(dir);
			if (dir === 'binaries') void refreshBinaries();
		}).then((fn) => unsubs.push(fn));

		return () => unsubs.forEach((fn) => fn());
	}, [refreshFiles, refreshBinaries]);

	// ── Submit ────────────────────────────────────────────────────────────────

	const onSubmit = async (data: FormValues) => {
		await saveSettings({
			schemaVersion: SETTINGS_SCHEMA_VERSION,
			inputDeviceId: data.inputDeviceId,
			outputDeviceId: data.outputDeviceId,
			pttShortcut: data.pttShortcut,
			ttsShortcut: data.ttsShortcut,
			ttsEngine: data.ttsEngine,
			activeTtsModelId: data.activeTtsModelId,
			ttsVoice: data.ttsVoice,
			activeSttModel: data.activeSttModel,
		});
		await settingsService.updateShortcuts(data.pttShortcut, data.ttsShortcut);
		setSaved(true);
		setTimeout(() => setSaved(false), 3000);
	};

	// ── Device dropdown options ───────────────────────────────────────────────

	const inputOptions = inputDevices.map((d) => ({ value: d.id, label: d.name }));
	const outputOptions = outputDevices.map((d) => ({ value: d.id, label: d.name }));

	return (
		<div className="min-h-screen bg-bg-base text-text">
			{/* Header */}
			<header className="border-b border-border px-6 py-4 flex items-center justify-between">
				<div className="flex items-center gap-3">
					<div className="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-primary">
						<svg
							className="w-4 h-4 text-white"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							strokeWidth={2}
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
							/>
						</svg>
					</div>
					<h1 className="text-lg font-semibold text-text">Vocyl Settings</h1>
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
						>
							<Controller
								name="pttShortcut"
								control={control}
								render={({ field }) => (
									<KeybindRecorder
										value={field.value}
										onChange={field.onChange}
									/>
								)}
							/>
						</SettingsRow>
						<SettingsRow
							label="Read Selection"
							description="Press once to start reading selected text. Press again to stop."
						>
							<Controller
								name="ttsShortcut"
								control={control}
								render={({ field }) => (
									<KeybindRecorder
										value={field.value}
										onChange={field.onChange}
									/>
								)}
							/>
						</SettingsRow>
					</SettingsSection>

					{/* ── Speech-to-Text ── */}
					<SettingsSection title="Speech-to-Text">
						<BinaryRow
							label="Whisper Runtime"
							description="Required to run speech recognition."
							isInstalled={whisperInstalled}
							dlKey="binaries/whisper"
							dlStates={dlStates}
							onDownload={() => modelService.downloadWhisper().catch(console.error)}
						/>

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
										onUse={(id) =>
											setValue('activeSttModel', id as SttModelSize)
										}
										onRefresh={(dir) => void refreshFiles(dir)}
									/>
								))}
							</div>
						</div>
					</SettingsSection>

					{/* ── Text-to-Speech ── */}
					<SettingsSection title="Text-to-Speech">
						{/* Engine selector */}
						<SettingsRow
							label="Engine"
							description="Kokoro is higher quality. Piper is lighter for low-end systems."
						>
							<Controller
								name="ttsEngine"
								control={control}
								render={({ field }) => (
									<div className="flex gap-3">
										{(['kokoro', 'piper'] as TtsEngine[]).map((engine) => (
											<label
												key={engine}
												className="flex items-center gap-2 cursor-pointer select-none"
											>
												<input
													type="radio"
													value={engine}
													checked={field.value === engine}
													onChange={() => {
														field.onChange(engine);
														setValue('activeTtsModelId', null);
													}}
													className="accent-primary"
												/>
												<span className="text-sm text-text capitalize">
													{engine}
												</span>
											</label>
										))}
									</div>
								)}
							/>
						</SettingsRow>

						{/* ── Kokoro section ── */}
						{ttsEngine === 'kokoro' && (
							<>
								<BinaryRow
									label="Kokoro English Model"
									description="Voice model + phoneme data (~90 MB). The inference runtime is built into the app."
									isInstalled={kokoroInstalled}
									dlKey="tts/kokoro"
									dlStates={dlStates}
									onDownload={() =>
										modelService.downloadKokoro().catch(console.error)
									}
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
												placeholder="Select voice…"
												className="w-56"
											/>
										)}
									/>
								</SettingsRow>
							</>
						)}

						{/* ── Piper section ── */}
						{ttsEngine === 'piper' && (
							<>
								<BinaryRow
									label="Piper Runtime"
									description="Required to run Piper text-to-speech."
									isInstalled={piperInstalled}
									dlKey="binaries/piper"
									dlStates={dlStates}
									onDownload={() => void modelService.downloadPiper()}
								/>

								<div className="px-4 py-2">
									<p className="text-xs font-semibold uppercase tracking-widest text-text-faint mb-2">
										Piper Voices
									</p>
									<div className="bg-bg-base rounded-lg border border-border divide-y divide-border">
										{PIPER_VOICES.map((voice) => (
											<ModelRow
												key={voice.id}
												id={voice.id}
												label={voice.label}
												sizeMb={voice.sizeMb}
												files={voice.files}
												dir="tts"
												downloadedFiles={ttsFiles}
												dlStates={dlStates}
												activeId={activeTtsModelId}
												onUse={(id) => setValue('activeTtsModelId', id)}
												onRefresh={(dir) => void refreshFiles(dir)}
											/>
										))}
									</div>
								</div>
							</>
						)}
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
									<svg
										className="w-4 h-4"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
										strokeWidth={2}
									>
										<path
											strokeLinecap="round"
											strokeLinejoin="round"
											d="M5 13l4 4L19 7"
										/>
									</svg>
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
