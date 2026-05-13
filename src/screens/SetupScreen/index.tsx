import { useForm } from 'react-hook-form';
import { DEFAULT_SETTINGS, SETTINGS_SCHEMA_VERSION } from '@models/settings';
import { useSettingsContext } from '@contexts/SettingsContext';
import Button from '@components/Button';

export default function SetupScreen() {
	const { saveSettings } = useSettingsContext();

	const {
		handleSubmit,
		formState: { isSubmitting },
	} = useForm();

	const onSubmit = async () => {
		await saveSettings({
			...DEFAULT_SETTINGS,
			schemaVersion: SETTINGS_SCHEMA_VERSION,
		});
	};

	return (
		<div className="min-h-screen bg-bg-base flex items-center justify-center p-6">
			<div className="w-full max-w-md">
				<div className="text-center mb-10">
					<div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary mb-4">
						<svg
							className="w-8 h-8 text-white"
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
					<h1 className="text-3xl font-bold text-text tracking-tight">
						Welcome to Vocyl
					</h1>
					<p className="mt-2 text-text-subtle text-sm">
						Voice-to-text and text-to-speech for your desktop. Configure everything from
						the Settings window.
					</p>
				</div>

				<form
					onSubmit={(e) => void handleSubmit(onSubmit)(e)}
					className="bg-bg-surface rounded-2xl border border-border p-6 space-y-4"
				>
					<div className="space-y-2 text-sm text-text-faint">
						<p>
							Default shortcuts will be set up automatically. You can change them and
							download TTS/STT models in Settings.
						</p>
						<ul className="space-y-1 pl-4 list-disc">
							<li>
								Push-to-talk:{' '}
								<span className="font-mono text-text-muted">⌃⇧F8</span>
							</li>
							<li>
								Read selection:{' '}
								<span className="font-mono text-text-muted">⌃⇧F9</span>
							</li>
						</ul>
					</div>

					<Button type="submit" loading={isSubmitting} fullWidth>
						Get Started
					</Button>
				</form>
			</div>
		</div>
	);
}
