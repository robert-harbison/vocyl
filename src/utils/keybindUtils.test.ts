import { describe, it, expect } from 'vitest';
import { keyboardEventToShortcut, KEY_NAMES } from './keybindUtils';

function makeEvent(
	overrides: Partial<
		Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'>
	>,
): Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'> {
	return {
		key: 'A',
		code: 'KeyA',
		ctrlKey: false,
		metaKey: false,
		shiftKey: false,
		altKey: false,
		...overrides,
	};
}

describe('keyboardEventToShortcut', () => {
	describe('returns null for modifier-only events', () => {
		it.each(['Control', 'Shift', 'Alt', 'Meta'])('returns null when key is "%s"', (key) => {
			expect(keyboardEventToShortcut(makeEvent({ key, ctrlKey: true }))).toBeNull();
		});
	});

	describe('returns null when no modifier is held', () => {
		it('returns null for a plain letter key with no modifiers', () => {
			expect(keyboardEventToShortcut(makeEvent({ key: 'A', code: 'KeyA' }))).toBeNull();
		});

		it('returns null for a special key with no modifiers', () => {
			expect(keyboardEventToShortcut(makeEvent({ key: 'F5', code: 'F5' }))).toBeNull();
		});
	});

	describe('Ctrl / Meta → CmdOrCtrl', () => {
		it('produces CmdOrCtrl when ctrlKey is held', () => {
			expect(keyboardEventToShortcut(makeEvent({ ctrlKey: true }))).toBe('CmdOrCtrl+A');
		});

		it('produces CmdOrCtrl when metaKey is held', () => {
			expect(keyboardEventToShortcut(makeEvent({ metaKey: true }))).toBe('CmdOrCtrl+A');
		});
	});

	describe('Shift modifier', () => {
		it('includes Shift in the output', () => {
			expect(keyboardEventToShortcut(makeEvent({ ctrlKey: true, shiftKey: true }))).toBe(
				'CmdOrCtrl+Shift+A',
			);
		});
	});

	describe('Alt modifier', () => {
		it('includes Alt in the output', () => {
			expect(keyboardEventToShortcut(makeEvent({ ctrlKey: true, altKey: true }))).toBe(
				'CmdOrCtrl+Alt+A',
			);
		});
	});

	describe('modifier ordering', () => {
		it('orders modifiers as CmdOrCtrl → Shift → Alt', () => {
			const result = keyboardEventToShortcut(
				makeEvent({ ctrlKey: true, shiftKey: true, altKey: true }),
			);
			expect(result).toBe('CmdOrCtrl+Shift+Alt+A');
		});
	});

	describe('key code stripping', () => {
		it('strips the "Key" prefix from e.code (e.g. KeyA → A)', () => {
			expect(
				keyboardEventToShortcut(makeEvent({ key: 'a', code: 'KeyA', ctrlKey: true })),
			).toBe('CmdOrCtrl+A');
		});

		it('strips the "Digit" prefix from e.code (e.g. Digit9 → 9)', () => {
			// Shift+9 → key is "(" but code is "Digit9"
			expect(
				keyboardEventToShortcut(
					makeEvent({ key: '(', code: 'Digit9', ctrlKey: true, shiftKey: true }),
				),
			).toBe('CmdOrCtrl+Shift+9');
		});
	});

	describe('KEY_NAMES mapping', () => {
		it('maps Space character to "Space"', () => {
			expect(
				keyboardEventToShortcut(makeEvent({ key: ' ', code: 'Space', ctrlKey: true })),
			).toBe('CmdOrCtrl+Space');
		});

		it('maps ArrowUp to "Up"', () => {
			expect(
				keyboardEventToShortcut(
					makeEvent({ key: 'ArrowUp', code: 'ArrowUp', ctrlKey: true }),
				),
			).toBe('CmdOrCtrl+Up');
		});

		it('maps ArrowDown to "Down"', () => {
			expect(
				keyboardEventToShortcut(
					makeEvent({ key: 'ArrowDown', code: 'ArrowDown', ctrlKey: true }),
				),
			).toBe('CmdOrCtrl+Down');
		});

		it('maps Enter to "Return"', () => {
			expect(
				keyboardEventToShortcut(makeEvent({ key: 'Enter', code: 'Enter', ctrlKey: true })),
			).toBe('CmdOrCtrl+Return');
		});

		it('maps Escape to "Escape"', () => {
			expect(
				keyboardEventToShortcut(
					makeEvent({ key: 'Escape', code: 'Escape', ctrlKey: true }),
				),
			).toBe('CmdOrCtrl+Escape');
		});

		it('maps F1–F12 through unchanged', () => {
			for (let i = 1; i <= 12; i++) {
				const key = `F${String(i)}`;
				expect(keyboardEventToShortcut(makeEvent({ key, code: key, ctrlKey: true }))).toBe(
					`CmdOrCtrl+${key}`,
				);
			}
		});
	});

	describe('KEY_NAMES export', () => {
		it('exports KEY_NAMES as a record', () => {
			expect(typeof KEY_NAMES).toBe('object');
		});

		it('contains an entry for ArrowUp', () => {
			expect(KEY_NAMES.ArrowUp).toBe('Up');
		});
	});
});
