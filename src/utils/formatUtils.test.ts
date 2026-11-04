import { describe, it, expect } from 'vitest';
import { formatShortcut, DISPLAY_MAP } from './formatUtils';

describe('formatShortcut', () => {
	describe('modifier translations', () => {
		it('translates CmdOrCtrl to Ctrl', () => {
			expect(formatShortcut('CmdOrCtrl+A')).toBe('Ctrl+A');
		});

		it('translates Ctrl to Ctrl', () => {
			expect(formatShortcut('Ctrl+A')).toBe('Ctrl+A');
		});

		it('translates Control to Ctrl', () => {
			expect(formatShortcut('Control+A')).toBe('Ctrl+A');
		});

		it('translates Shift to Shift', () => {
			expect(formatShortcut('Shift+S')).toBe('Shift+S');
		});

		it('translates Alt to Alt', () => {
			expect(formatShortcut('Alt+F4')).toBe('Alt+F4');
		});

		it('translates Meta to Cmd', () => {
			expect(formatShortcut('Meta+Space')).toBe('Cmd+Space');
		});

		it('translates Super to Win', () => {
			expect(formatShortcut('Super+D')).toBe('Win+D');
		});

		it('translates Win to Win', () => {
			expect(formatShortcut('Win+D')).toBe('Win+D');
		});
	});

	describe('key translations', () => {
		it('translates Return to Enter', () => {
			expect(formatShortcut('CmdOrCtrl+Return')).toBe('Ctrl+Enter');
		});

		it('translates Space to Space (passthrough)', () => {
			expect(formatShortcut('CmdOrCtrl+Space')).toBe('Ctrl+Space');
		});

		it('keeps unknown keys as-is', () => {
			expect(formatShortcut('CmdOrCtrl+F5')).toBe('Ctrl+F5');
			expect(formatShortcut('CmdOrCtrl+Z')).toBe('Ctrl+Z');
		});
	});

	describe('multi-modifier shortcuts', () => {
		it('handles CmdOrCtrl+Shift+key', () => {
			expect(formatShortcut('CmdOrCtrl+Shift+A')).toBe('Ctrl+Shift+A');
		});

		it('handles CmdOrCtrl+Shift+Alt+key', () => {
			expect(formatShortcut('CmdOrCtrl+Shift+Alt+K')).toBe('Ctrl+Shift+Alt+K');
		});
	});

	describe('edge cases', () => {
		it('returns the key unchanged when there are no modifiers to translate', () => {
			expect(formatShortcut('A')).toBe('A');
		});

		it('handles an empty string without throwing', () => {
			expect(formatShortcut('')).toBe('');
		});
	});

	describe('DISPLAY_MAP', () => {
		it('exports DISPLAY_MAP as a record', () => {
			expect(typeof DISPLAY_MAP).toBe('object');
		});

		it('maps CmdOrCtrl to Ctrl', () => {
			expect(DISPLAY_MAP.CmdOrCtrl).toBe('Ctrl');
		});
	});
});
