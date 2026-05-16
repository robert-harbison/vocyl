import { useState, useEffect, useRef, useCallback, type JSX } from 'react';
import { clsx } from 'clsx';
import { settingsService } from '@services/settingsService';
import { formatShortcut } from '@utils/formatUtils';
import { keyboardEventToShortcut } from '@utils/keybindUtils';

interface KeybindRecorderProps {
	value: string;
	onChange: (shortcut: string) => void;
	className?: string;
}

export default function KeybindRecorder({
	value,
	onChange,
	className,
}: KeybindRecorderProps): JSX.Element {
	const [recording, setRecording] = useState(false);
	const btnRef = useRef<HTMLButtonElement>(null);

	const stopRecording = useCallback(() => {
		setRecording(false);
		void settingsService.resumeShortcuts().catch((err: unknown) => {
			console.error(err);
		});
	}, []);

	useEffect(() => {
		if (!recording) return;

		void settingsService.suspendShortcuts().catch((err: unknown) => {
			console.error(err);
		});

		const onKeyDown = (e: KeyboardEvent) => {
			e.preventDefault();
			e.stopPropagation();
			if (e.key === 'Escape') {
				stopRecording();
				return;
			}
			const shortcut = keyboardEventToShortcut(e);
			if (shortcut) {
				onChange(shortcut);
				stopRecording();
			}
		};

		window.addEventListener('keydown', onKeyDown, true);
		return () => {
			window.removeEventListener('keydown', onKeyDown, true);
		};
	}, [recording, onChange, stopRecording]);

	return (
		<button
			ref={btnRef}
			type="button"
			className={clsx(
				'inline-flex items-center justify-center min-w-25 rounded-lg px-3 py-1.5',
				'text-sm font-mono transition border focus:outline-none focus:ring-2 focus:ring-primary',
				recording ?
					'border-primary bg-primary/10 text-primary cursor-text'
				:	'border-border-input bg-bg-elevated text-text-muted hover:border-border cursor-pointer',
				className,
			)}
			onClick={() => {
				setRecording(true);
				btnRef.current?.focus();
			}}
			onBlur={stopRecording}
			title={
				recording ? 'Press a combination - Esc to cancel' : 'Click to record a new shortcut'
			}
		>
			{recording ?
				<span className="text-xs animate-pulse">Press keys...</span>
			:	<span>{formatShortcut(value)}</span>}
		</button>
	);
}
