import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from './App';

vi.mock('@services/logService');

vi.mock('@contexts/SettingsContext', () => ({
	useSettingsContext: vi.fn(),
}));

vi.mock('@screens/SettingsScreen', () => ({
	default: () => <div data-testid="settings-screen">SettingsScreen</div>,
}));

vi.mock('@screens/SetupScreen', () => ({
	default: () => <div data-testid="setup-screen">SetupScreen</div>,
}));

vi.mock('@components/Toast', () => ({
	default: ({ message, onDismiss }: { message: string; onDismiss: () => void }) => (
		<div data-testid="toast">
			{message}
			<button onClick={onDismiss}>dismiss</button>
		</div>
	),
}));

vi.mock('@tauri-apps/api/event');

import { useSettingsContext } from '@contexts/SettingsContext';
import { listen } from '@tauri-apps/api/event';
import type { VocylSettings } from '@models/settings';

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockListen = vi.mocked(listen);

interface ContextStub {
	loading: boolean;
	exists: boolean | null;
	settings: VocylSettings;
	saveSettings: ReturnType<typeof vi.fn>;
	setExists: ReturnType<typeof vi.fn>;
	error: string | null;
}

function makeContext(overrides: Partial<ContextStub> = {}): ReturnType<typeof useSettingsContext> {
	return {
		loading: false,
		exists: true,
		settings: {} as VocylSettings,
		saveSettings: vi.fn(),
		setExists: vi.fn(),
		error: null,
		...overrides,
	} as ReturnType<typeof useSettingsContext>;
}

beforeEach(() => {
	vi.clearAllMocks();
	mockListen.mockResolvedValue(vi.fn());
});

describe('App', () => {
	describe('loading state', () => {
		it('renders Loader while settings are loading', () => {
			mockUseSettingsContext.mockReturnValue(makeContext({ loading: true }));
			const { container } = render(<App />);
			expect(container.querySelector('.animate-spin')).toBeInTheDocument();
		});

		it('does not render screens while loading', () => {
			mockUseSettingsContext.mockReturnValue(makeContext({ loading: true }));
			render(<App />);
			expect(screen.queryByTestId('settings-screen')).not.toBeInTheDocument();
			expect(screen.queryByTestId('setup-screen')).not.toBeInTheDocument();
		});
	});

	describe('screen selection', () => {
		it('renders SettingsScreen when settings exist', () => {
			mockUseSettingsContext.mockReturnValue(makeContext({ loading: false, exists: true }));
			render(<App />);
			expect(screen.getByTestId('settings-screen')).toBeInTheDocument();
			expect(screen.queryByTestId('setup-screen')).not.toBeInTheDocument();
		});

		it('renders SetupScreen when settings do not exist', () => {
			mockUseSettingsContext.mockReturnValue(makeContext({ loading: false, exists: false }));
			render(<App />);
			expect(screen.getByTestId('setup-screen')).toBeInTheDocument();
			expect(screen.queryByTestId('settings-screen')).not.toBeInTheDocument();
		});
	});

	describe('single-instance event', () => {
		it('registers a listener for single-instance-attempted', async () => {
			mockUseSettingsContext.mockReturnValue(makeContext());
			render(<App />);
			await waitFor(() => {
				expect(mockListen).toHaveBeenCalledWith(
					'single-instance-attempted',
					expect.any(Function),
				);
			});
		});

		it('shows Toast when single-instance-attempted event fires', () => {
			mockUseSettingsContext.mockReturnValue(makeContext());
			let capturedHandler: (() => void) | null = null;
			mockListen.mockImplementation((_event, cb) => {
				capturedHandler = cb as () => void;
				return Promise.resolve(vi.fn());
			});

			render(<App />);
			expect(screen.queryByTestId('toast')).not.toBeInTheDocument();

			// listen() is called synchronously in the effect, so capturedHandler is set after render.
			act(() => {
				capturedHandler!();
			});

			expect(screen.getByTestId('toast')).toBeInTheDocument();
			expect(screen.getByText('Vocyl is already running.')).toBeInTheDocument();
		});

		it('hides Toast when onDismiss is called', async () => {
			mockUseSettingsContext.mockReturnValue(makeContext());
			let capturedHandler: (() => void) | null = null;
			mockListen.mockImplementation((_event, cb) => {
				capturedHandler = cb as () => void;
				return Promise.resolve(vi.fn());
			});

			const user = userEvent.setup();
			render(<App />);

			act(() => {
				capturedHandler!();
			});

			expect(screen.getByTestId('toast')).toBeInTheDocument();
			await user.click(screen.getByRole('button', { name: 'dismiss' }));
			expect(screen.queryByTestId('toast')).not.toBeInTheDocument();
		});

		it('cleans up the event listener on unmount', async () => {
			mockUseSettingsContext.mockReturnValue(makeContext());
			const mockUnlisten = vi.fn();
			mockListen.mockResolvedValue(mockUnlisten);

			const { unmount } = render(<App />);
			unmount();
			await waitFor(() => {
				expect(mockUnlisten).toHaveBeenCalledOnce();
			});
		});
	});
});
