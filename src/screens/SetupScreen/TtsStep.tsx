import type { JSX } from 'react';
import { AlertTriangle } from 'lucide-react';
import Button from '@components/Button';
import BinaryRow from '@screens/SettingsScreen/BinaryRow';
import type { DownloadStates } from '@screens/SettingsScreen/ModelRow';

interface TtsStepProps {
	kokoroInstalled: boolean;
	dlStates: DownloadStates;
	onDownload: () => Promise<void>;
	onDelete: () => Promise<void>;
	onNext: () => void;
	onSkip: () => void;
}

export default function TtsStep({
	kokoroInstalled,
	dlStates,
	onDownload,
	onDelete,
	onNext,
	onSkip,
}: TtsStepProps): JSX.Element {
	return (
		<div className="space-y-6">
			<div className="text-center">
				<h2 className="text-2xl font-bold text-text tracking-tight">
					Text-to-Speech Model
				</h2>
				<p className="mt-1 text-sm text-text-subtle max-w-sm mx-auto">
					Download the Kokoro voice synthesis model to enable &ldquo;Read
					Selection&rdquo;.
				</p>
			</div>

			<div className="bg-bg-surface rounded-xl border border-border divide-y divide-border">
				<BinaryRow
					label="Kokoro English Model"
					description="Neural voice model + phoneme data (~90 MB). The inference runtime is built into the app."
					isInstalled={kokoroInstalled}
					dlKey="tts/kokoro"
					dlStates={dlStates}
					onDownload={onDownload}
					onDelete={onDelete}
				/>
			</div>

			{/* GPL-3 notice */}
			<div className="flex items-start gap-2 rounded-lg bg-bg-elevated border border-border px-3 py-2.5 text-xs text-text-faint">
				<AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5 text-text-muted" />
				<p>
					<span className="font-semibold text-text-muted">
						Third-party GPL-3.0 software:
					</span>{' '}
					By downloading, you are installing the Kokoro voice model which is licensed
					under the GNU General Public License v3 (GPL-3.0). See{' '}
					<span className="font-mono">THIRD_PARTY_LICENSES.md</span> for full licence
					text.
				</p>
			</div>

			<div className="flex items-center justify-between">
				<button
					type="button"
					onClick={onSkip}
					className="text-sm text-text-faint hover:text-text transition-colors"
				>
					Skip for now
				</button>
				<Button onClick={onNext}>Continue →</Button>
			</div>
		</div>
	);
}
