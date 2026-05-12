declare const __APP_VERSION__: string;

import { useState } from 'react';
import { SETTINGS_SCHEMA_VERSION } from '../types/settings';
import type { VocylSettings } from '../types/settings';

interface Props {
	settings: VocylSettings;
	onSave: (settings: VocylSettings) => Promise<void>;
}

interface FormState {
	testProperty: string;
}

export default function SettingsScreen({ settings, onSave }: Props) {
	const [form, setForm] = useState<FormState>({
		testProperty: settings.testProperty,
	});
	const [saving, setSaving] = useState(false);
	const [saveError, setSaveError] = useState<string | null>(null);
	const [saved, setSaved] = useState(false);

	const handleChange =
		(field: keyof FormState) =>
		(e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
			setForm((prev) => ({ ...prev, [field]: e.target.value }));
			setSaved(false);
		};

	const handleSubmit = async (e: React.SubmitEvent<HTMLFormElement>) => {
		e.preventDefault();
		setSaving(true);
		setSaveError(null);
		try {
			await onSave({
				schemaVersion: SETTINGS_SCHEMA_VERSION,
				testProperty: form.testProperty,
			});
			setSaved(true);
		} catch (err) {
			setSaveError(String(err));
		} finally {
			setSaving(false);
		}
	};

	return (
		<div className="min-h-screen bg-gray-950 text-white">
			{/* Header */}
			<header className="border-b border-gray-800 px-6 py-4 flex items-center justify-between">
				<div className="flex items-center gap-3">
					<div className="inline-flex items-center justify-center w-8 h-8 rounded-lg bg-indigo-600">
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
					<h1 className="text-lg font-semibold text-white">
						Vocyl Settings
					</h1>
				</div>
				<span className="text-xs text-gray-500 font-mono bg-gray-800 px-2 py-1 rounded">
					v{__APP_VERSION__}
				</span>
			</header>

			{/* Content */}
			<main className="max-w-2xl mx-auto px-6 py-8">
				<form
					onSubmit={(e) => void handleSubmit(e)}
					className="space-y-8"
				>
					{/* General section */}
					<section>
						<h2 className="text-xs font-semibold uppercase tracking-widest text-gray-500 mb-4">
							General
						</h2>
						<div className="bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
							<SettingsRow
								label="Test Property"
								description="Placeholder field for future configuration."
							>
								<input
									type="text"
									value={form.testProperty}
									onChange={handleChange('testProperty')}
									placeholder="Enter a value…"
									className="w-56 rounded-lg bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
								/>
							</SettingsRow>
						</div>
					</section>

					{/* Audio section (demo / placeholder) */}
					<section>
						<h2 className="text-xs font-semibold uppercase tracking-widest text-gray-500 mb-4">
							Audio
						</h2>
						<div className="bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
							<SettingsRow
								label="Input Device"
								description="Microphone used for push-to-talk."
							>
								<select
									disabled
									className="w-56 rounded-lg bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-500 focus:outline-none transition cursor-not-allowed"
								>
									<option>Default Microphone</option>
								</select>
							</SettingsRow>
							<SettingsRow
								label="Noise Suppression"
								description="Reduce background noise during recording."
							>
								<TogglePlaceholder />
							</SettingsRow>
						</div>
					</section>

					{/* Shortcut section (demo / placeholder) */}
					<section>
						<h2 className="text-xs font-semibold uppercase tracking-widest text-gray-500 mb-4">
							Shortcut
						</h2>
						<div className="bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
							<SettingsRow
								label="Push-to-Talk Key"
								description="Hold this key combination to activate the microphone."
							>
								<kbd className="inline-flex items-center gap-1 rounded bg-gray-800 border border-gray-700 px-2 py-1 text-xs text-gray-300 font-mono">
									⌃⇧Space
								</kbd>
							</SettingsRow>
						</div>
					</section>

					{/* Save feedback + button */}
					{saveError && (
						<p className="text-xs text-red-400 bg-red-900/20 border border-red-800 rounded-lg px-3 py-2">
							{saveError}
						</p>
					)}

					<div className="flex items-center justify-end gap-3">
						{saved && (
							<span className="text-sm text-green-400 flex items-center gap-1.5">
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
						<button
							type="submit"
							disabled={saving}
							className="py-2 px-5 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-950"
						>
							{saving ? 'Saving…' : 'Save Settings'}
						</button>
					</div>
				</form>
			</main>
		</div>
	);
}

// ── Small sub-components ──────────────────────────────────────────────────────

function SettingsRow({
	label,
	description,
	children,
}: {
	label: string;
	description: string;
	children: React.ReactNode;
}) {
	return (
		<div className="flex items-center justify-between px-4 py-3 gap-4">
			<div className="min-w-0">
				<p className="text-sm font-medium text-gray-200">{label}</p>
				<p className="text-xs text-gray-500 mt-0.5">{description}</p>
			</div>
			<div className="shrink-0">{children}</div>
		</div>
	);
}

function TogglePlaceholder() {
	return (
		<div
			className="w-10 h-6 rounded-full bg-gray-700 opacity-50 cursor-not-allowed"
			title="Coming soon"
		/>
	);
}
