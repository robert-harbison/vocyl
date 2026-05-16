import type { JSX } from 'react';
import Button from '@components/Button';
import KeybindRecorder from '@components/KeybindRecorder';
import SettingsRow from '@screens/SettingsScreen/SettingsRow';
import SettingsSection from '@screens/SettingsScreen/SettingsSection';

interface KeybindsStepProps {
	pttShortcut: string;
	ttsShortcut: string;
	onPttChange: (v: string) => void;
	onTtsChange: (v: string) => void;
	onFinish: () => void;
	onSkip: () => void;
}

export default function KeybindsStep({
	pttShortcut,
	ttsShortcut,
	onPttChange,
	onTtsChange,
	onFinish,
	onSkip,
}: KeybindsStepProps): JSX.Element {
	const conflict = pttShortcut === ttsShortcut;

	return (
		<div className="space-y-6">
			<div className="text-center">
				<h2 className="text-2xl font-bold text-text tracking-tight">Keyboard Shortcuts</h2>
				<p className="mt-1 text-sm text-text-subtle max-w-sm mx-auto">
					Click a recorder and press your desired key combination. These can be changed
					any time in Settings.
				</p>
			</div>

			<SettingsSection title="Shortcuts">
				<SettingsRow
					label="Push-to-Talk"
					description="Hold to record. Text is typed at your cursor on release."
					error={conflict ? 'Shortcuts must be different' : undefined}
				>
					<KeybindRecorder value={pttShortcut} onChange={onPttChange} />
				</SettingsRow>
				<SettingsRow
					label="Read Selection"
					description="Press once to start reading selected text. Press again to stop."
					error={conflict ? 'Shortcuts must be different' : undefined}
				>
					<KeybindRecorder value={ttsShortcut} onChange={onTtsChange} />
				</SettingsRow>
			</SettingsSection>

			<div className="flex items-center justify-between">
				<button
					type="button"
					onClick={onSkip}
					className="text-sm text-text-faint hover:text-text transition-colors"
				>
					Skip for now
				</button>
				<Button onClick={onFinish} disabled={conflict}>
					Finish Setup →
				</Button>
			</div>
		</div>
	);
}
