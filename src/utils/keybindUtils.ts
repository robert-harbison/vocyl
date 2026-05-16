export const KEY_NAMES: Record<string, string> = {
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

/**
 * Converts a KeyboardEvent into a Tauri-style shortcut string such as
 * "CmdOrCtrl+Shift+A", or returns null if the event should be ignored
 * (modifier-only keydown, or no modifier held).
 */
export function keyboardEventToShortcut(
	e: Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'>,
): string | null {
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
