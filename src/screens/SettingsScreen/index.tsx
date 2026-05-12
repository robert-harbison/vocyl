declare const __APP_VERSION__: string;

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { SETTINGS_SCHEMA_VERSION } from '@models/settings';
import { useSettingsContext } from '@contexts/SettingsContext';
import Button from '@components/Button';
import TextField from '@components/TextField';
import SettingsRow from './SettingsRow';
import SettingsSection from './SettingsSection';

interface FormValues {
	testProperty: string;
}

export default function SettingsScreen() {
	const { settings, saveSettings } = useSettingsContext();
	const [saved, setSaved] = useState(false);

	const {
		register,
		handleSubmit,
		formState: { isSubmitting, errors },
	} = useForm<FormValues>({
		defaultValues: { testProperty: settings.testProperty },
	});

	const onSubmit = async (data: FormValues) => {
		await saveSettings({
			schemaVersion: SETTINGS_SCHEMA_VERSION,
			testProperty: data.testProperty,
		});
		setSaved(true);
	};

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
					<SettingsSection title="General">
						<SettingsRow
							label="Test Property"
							description="Placeholder field for future configuration."
						>
							<TextField
								{...register('testProperty', {
									required: 'This field is required',
								})}
								type="text"
								placeholder="Enter a value..."
								error={errors.testProperty?.message}
								className="w-56"
							/>
						</SettingsRow>
					</SettingsSection>

					<SettingsSection title="Audio">
						<SettingsRow
							label="Input Device"
							description="Microphone used for push-to-talk."
						>
							<select
								disabled
								className="w-56 rounded-lg bg-bg-elevated border border-border-input px-3 py-1.5 text-sm text-text-faint focus:outline-none transition cursor-not-allowed"
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
					</SettingsSection>

					<SettingsSection title="Shortcut">
						<SettingsRow
							label="Push-to-Talk Key"
							description="Hold this key combination to activate the microphone."
						>
							<kbd className="inline-flex items-center gap-1 rounded bg-bg-elevated border border-border-input px-2 py-1 text-xs text-text-muted font-mono">
								⌃⇧Space
							</kbd>
						</SettingsRow>
					</SettingsSection>

					<div className="flex items-center justify-end gap-3">
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
				</form>
			</main>
		</div>
	);
}

// ── Small sub-components ──────────────────────────────────────────────────────

function TogglePlaceholder() {
	return (
		<div
			className="w-10 h-6 rounded-full bg-bg-elevated opacity-50 cursor-not-allowed"
			title="Coming soon"
		/>
	);
}
