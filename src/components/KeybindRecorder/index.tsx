import { useState, useEffect, useRef, useCallback, type JSX } from 'react';
import { clsx } from 'clsx';

interface KeybindRecorderProps {
	value: string;
	onChange: (shortcut: string) => void;
	className?: string;
}

const KEY_NAMES: Record<string, string> = {
	' ': 'Space',
	ArrowUp: 'Up',
	ArrowDown: 'Down',
	ArrowLeft: 'Left',
	ArrowRight: 'Right',
	Delete: 'Delete',
	Backspace: 'Backspace',
	Enter: 'Return',
	Tab: 'Tab',
	Escape: 'Escape',
	Home: 'Home',
	End: 'End',
	PageUp: 'PageUp',
	PageDown: 'PageDown',
	Insert: 'Insert',
	F1: 'F1',
	F2: 'F2',
	F3: 'F3',
	F4: 'F4',
	F5: 'F5',
	F6: 'F6',
	F7: 'F7',
	F8: 'F8',
	F9: 'F9',
	F10: 'F10',
	F11: 'F11',
	F12: 'F12',
};

const DISPLAY_MAP: Record<string, string> = {
	CmdOrCtrl: '⌃',
	Ctrl: '⌃',
	Control: '⌃',
	Shift: '⇧',
	Alt: '⌥',
	Meta: '⌘',
	Super: '⊞',
	Win: '⊞',
	Space: 'Space',
	Return: '↵',
};

export function formatShortcut(shortcut: string): string {
	return shortcut
		.split('+')
		.map((part) => DISPLAY_MAP[part] ?? part)
		.join('');
}

function keyboardEventToShortcut(e: KeyboardEvent): string | null {
	if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return null;

	const mods: string[] = [];
	if (e.ctrlKey || e.metaKey) mods.push('CmdOrCtrl');
	if (e.shiftKey) mods.push('Shift');
	if (e.altKey) mods.push('Alt');

	if (mods.length === 0) return null;

	// Use e.code (physical key) instead of e.key so that Shift+9 records "9"
	// rather than the shifted character "(". Strip "Key" prefix (e.g. "KeyA" → "A")
	// and "Digit" prefix (e.g. "Digit9" → "9") from code values.
	const rawCode = e.code.replace(/^Key/, '').replace(/^Digit/, '');
	const key = KEY_NAMES[e.key] ?? rawCode;
	return [...mods, key].join('+');
}

export default function KeybindRecorder({
	value,
	onChange,
	className,
}: KeybindRecorderProps): JSX.Element {
	const [recording, setRecording] = useState(false);
	const btnRef = useRef<HTMLButtonElement>(null);

	const stopRecording = useCallback(() => setRecording(false), []);

	useEffect(() => {
		if (!recording) return;

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
		return () => window.removeEventListener('keydown', onKeyDown, true);
	}, [recording, onChange, stopRecording]);

	return (
		<button
			ref={btnRef}
			type="button"
			className={clsx(
				'inline-flex items-center justify-center min-w-[100px] rounded-lg px-3 py-1.5',
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
				recording ? 'Press a combination — Esc to cancel' : 'Click to record a new shortcut'
			}
		>
			{recording ?
				<span className="text-xs animate-pulse">Press keys…</span>
			:	<span>{formatShortcut(value)}</span>}
		</button>
	);
}
