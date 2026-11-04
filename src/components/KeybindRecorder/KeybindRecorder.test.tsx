import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import KeybindRecorder from './index';
import { settingsService } from '@services/settingsService';

vi.mock('@services/settingsService', () => ({
	settingsService: {
		suspendShortcuts: vi.fn(),
		resumeShortcuts: vi.fn(),
	},
}));

const mockSuspend = vi.mocked(settingsService.suspendShortcuts);
const mockResume = vi.mocked(settingsService.resumeShortcuts);

beforeEach(() => {
	vi.clearAllMocks();
	mockSuspend.mockResolvedValue(undefined);
	mockResume.mockResolvedValue(undefined);
});

function getButton() {
	return screen.getByRole('button');
}

function fireKeyDown(overrides: Partial<KeyboardEventInit> = {}) {
	act(() => {
		window.dispatchEvent(
			new KeyboardEvent('keydown', {
				bubbles: true,
				cancelable: true,
				key: 'A',
				code: 'KeyA',
				ctrlKey: false,
				metaKey: false,
				shiftKey: false,
				altKey: false,
				...overrides,
			}),
		);
	});
}

describe('KeybindRecorder', () => {
	describe('initial render', () => {
		it('displays the formatted shortcut value', () => {
			render(<KeybindRecorder value="CmdOrCtrl+Shift+A" onChange={vi.fn()} />);
			expect(screen.getByText('Ctrl+Shift+A')).toBeInTheDocument();
		});

		it('is not in recording mode on first render', () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			expect(screen.queryByText('Press keys...')).not.toBeInTheDocument();
		});

		it('shows the idle title on the button', () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			expect(getButton()).toHaveAttribute('title', 'Click to record a new shortcut');
		});

		it('applies a custom className to the button', () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} className="my-class" />);
			expect(getButton()).toHaveClass('my-class');
		});
	});

	describe('entering recording mode', () => {
		it('shows "Press keys..." after clicking the button', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			expect(screen.getByText('Press keys...')).toBeInTheDocument();
		});

		it('hides the formatted shortcut while recording', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			expect(screen.queryByText('Ctrl+A')).not.toBeInTheDocument();
		});

		it('updates the button title while recording', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			expect(getButton()).toHaveAttribute('title', 'Press a combination - Esc to cancel');
		});

		it('calls suspendShortcuts when recording starts', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			expect(mockSuspend).toHaveBeenCalledOnce();
		});
	});

	describe('recording a shortcut', () => {
		it('calls onChange with the recorded shortcut', async () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'S', code: 'KeyS', ctrlKey: true });
			expect(onChange).toHaveBeenCalledOnce();
			expect(onChange).toHaveBeenCalledWith('CmdOrCtrl+S');
		});

		it('exits recording mode after a valid shortcut is pressed', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'S', code: 'KeyS', ctrlKey: true });
			expect(screen.queryByText('Press keys...')).not.toBeInTheDocument();
		});

		it('calls resumeShortcuts after a valid shortcut is recorded', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'S', code: 'KeyS', ctrlKey: true });
			expect(mockResume).toHaveBeenCalledOnce();
		});

		it('does not call onChange for a modifier-only keydown', async () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'Control', code: 'ControlLeft', ctrlKey: true });
			expect(onChange).not.toHaveBeenCalled();
			// still in recording mode
			expect(screen.getByText('Press keys...')).toBeInTheDocument();
		});

		it('does not call onChange for a keydown with no modifier', async () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'A', code: 'KeyA' });
			expect(onChange).not.toHaveBeenCalled();
		});
	});

	describe('cancelling with Escape', () => {
		it('exits recording mode when Escape is pressed', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'Escape', code: 'Escape', ctrlKey: false });
			expect(screen.queryByText('Press keys...')).not.toBeInTheDocument();
		});

		it('does not call onChange when Escape is pressed', async () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'Escape', code: 'Escape' });
			expect(onChange).not.toHaveBeenCalled();
		});

		it('calls resumeShortcuts when Escape cancels recording', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'Escape', code: 'Escape' });
			expect(mockResume).toHaveBeenCalledOnce();
		});
	});

	describe('blur cancels recording', () => {
		it('exits recording mode when the button loses focus', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			act(() => {
				getButton().blur();
			});
			expect(screen.queryByText('Press keys...')).not.toBeInTheDocument();
		});

		it('calls resumeShortcuts on blur', async () => {
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={vi.fn()} />);
			await userEvent.click(getButton());
			act(() => {
				getButton().blur();
			});
			expect(mockResume).toHaveBeenCalledOnce();
		});
	});

	describe('keydown listener lifecycle', () => {
		it('does not listen to keydown events when not recording', () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			fireKeyDown({ key: 'S', code: 'KeyS', ctrlKey: true });
			expect(onChange).not.toHaveBeenCalled();
		});

		it('removes the keydown listener after recording stops', async () => {
			const onChange = vi.fn();
			render(<KeybindRecorder value="CmdOrCtrl+A" onChange={onChange} />);
			await userEvent.click(getButton());
			fireKeyDown({ key: 'S', code: 'KeyS', ctrlKey: true }); // records, stops
			onChange.mockClear();
			fireKeyDown({ key: 'Z', code: 'KeyZ', ctrlKey: true }); // should be ignored
			expect(onChange).not.toHaveBeenCalled();
		});
	});
});
