import type { JSX } from 'react';
import Button from '@components/Button';

interface WelcomeStepProps {
	onNext: () => void;
}

export default function WelcomeStep({ onNext }: WelcomeStepProps): JSX.Element {
	return (
		<div className="text-center space-y-6">
			<div>
				<h1 className="text-3xl font-bold text-text tracking-tight">Welcome to Vocyl</h1>
				<p className="mt-2 text-text-subtle text-sm max-w-sm mx-auto">
					Voice-to-text and text-to-speech for your desktop. Let&apos;s get you set up in
					a few quick steps.
				</p>
			</div>

			<div className="bg-bg-surface rounded-2xl border border-border p-5 text-left space-y-3">
				<p className="text-sm font-medium text-text">What we&apos;ll set up:</p>
				<ul className="space-y-2 text-sm text-text-faint">
					<li className="flex items-start gap-2">
						<span className="mt-0.5 shrink-0 w-4 h-4 rounded-full bg-primary/20 text-primary text-[10px] font-bold flex items-center justify-center">
							1
						</span>
						Text-to-speech voice model (Kokoro)
					</li>
					<li className="flex items-start gap-2">
						<span className="mt-0.5 shrink-0 w-4 h-4 rounded-full bg-primary/20 text-primary text-[10px] font-bold flex items-center justify-center">
							2
						</span>
						Speech-to-text transcription model (Whisper)
					</li>
					<li className="flex items-start gap-2">
						<span className="mt-0.5 shrink-0 w-4 h-4 rounded-full bg-primary/20 text-primary text-[10px] font-bold flex items-center justify-center">
							3
						</span>
						Keyboard shortcuts for push-to-talk &amp; read selection
					</li>
				</ul>
			</div>

			<Button fullWidth onClick={onNext}>
				Get Started →
			</Button>
		</div>
	);
}
