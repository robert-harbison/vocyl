import type { JSX, ReactNode } from 'react';
import logoSquare from '@assets/IconLogoColor_Square.png';

type WizardStep = 'welcome' | 'tts' | 'stt' | 'keybinds' | 'finish';

const STEPS: WizardStep[] = ['welcome', 'tts', 'stt', 'keybinds', 'finish'];

interface WizardShellProps {
	step: WizardStep;
	children: ReactNode;
}

export default function WizardShell({ step, children }: WizardShellProps): JSX.Element {
	const currentIndex = STEPS.indexOf(step);
	// Don't show progress on welcome or finish
	const showProgress = step !== 'welcome' && step !== 'finish';

	return (
		<div className="min-h-screen bg-bg-base flex items-center justify-center p-6">
			<div className="w-full max-w-lg">
				{/* Logo */}
				<div className="flex justify-center mb-8">
					<img src={logoSquare} className="h-14" alt="Vocyl Logo" />
				</div>

				{/* Step progress dots */}
				{showProgress && (
					<div className="flex justify-center gap-2 mb-6">
						{[1, 2, 3].map((dot, i) => (
							<div
								key={dot}
								className={`w-2 h-2 rounded-full transition-colors ${
									i < currentIndex - 1 ? 'bg-primary'
									: i === currentIndex - 1 ? 'bg-primary'
									: 'bg-border'
								}`}
							/>
						))}
					</div>
				)}

				{children}
			</div>
		</div>
	);
}
