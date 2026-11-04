import type { JSX } from 'react';
import { CheckCircle2 } from 'lucide-react';
import Button from '@components/Button';

interface FinishStepProps {
	pttShortcut: string;
	ttsShortcut: string;
	onClose: () => void;
	isSaving: boolean;
}

export default function FinishStep({
	pttShortcut,
	ttsShortcut,
	onClose,
	isSaving,
}: FinishStepProps): JSX.Element {
	return (
		<div className="text-center space-y-6">
			<div>
				<div className="flex justify-center mb-4">
					<CheckCircle2 className="w-16 h-16 text-success" />
				</div>
				<h2 className="text-2xl font-bold text-text tracking-tight">
					You&apos;re all set!
				</h2>
				<p className="mt-1 text-sm text-text-subtle max-w-sm mx-auto">
					Vocyl is running in the background and ready to use.
				</p>
			</div>

			<div className="bg-bg-surface rounded-xl border border-border p-5 text-left space-y-3 text-sm text-text-faint">
				<p className="font-semibold text-text-muted text-xs uppercase tracking-wider">
					Quick reference
				</p>
				<ul className="space-y-2">
					<li className="flex items-start gap-2">
						<span className="mt-0.5">🎙</span>
						<span>
							Hold{' '}
							<kbd className="font-mono text-xs bg-bg-elevated border border-border px-1.5 py-0.5 rounded">
								{pttShortcut}
							</kbd>{' '}
							to record. Release to type at your cursor.
						</span>
					</li>
					<li className="flex items-start gap-2">
						<span className="mt-0.5">🔊</span>
						<span>
							Press{' '}
							<kbd className="font-mono text-xs bg-bg-elevated border border-border px-1.5 py-0.5 rounded">
								{ttsShortcut}
							</kbd>{' '}
							to read selected text aloud.
						</span>
					</li>
					<li className="flex items-start gap-2">
						<span className="mt-0.5">⚙️</span>
						<span>
							Right-click the{' '}
							<span className="font-medium text-text-muted">system tray icon</span>{' '}
							(bottom-right of your taskbar) to open Settings or quit Vocyl.
						</span>
					</li>
				</ul>
			</div>

			<Button fullWidth onClick={onClose} loading={isSaving}>
				Open Settings
			</Button>
		</div>
	);
}
