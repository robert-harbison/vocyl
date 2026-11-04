import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SetupScreen from './index';

// ── Service mocks ─────────────────────────────────────────────────────────────

vi.mock('@services/logService');
vi.mock('@tauri-apps/api/event');

vi.mock('@contexts/SettingsContext', () => ({
	useSettingsContext: vi.fn(),
}));

vi.mock('@services/modelService', () => ({
	modelService: {
		listDownloadedFiles: vi.fn(),
		isKokoroInstalled: vi.fn(),
		downloadKokoro: vi.fn(),
		deleteKokoro: vi.fn(),
		downloadFile: vi.fn(),
		deleteFile: vi.fn(),
	},
}));

vi.mock('@services/settingsService', () => ({
	settingsService: {
		updateShortcuts: vi.fn(),
		suspendShortcuts: vi.fn(),
		resumeShortcuts: vi.fn(),
	},
}));

vi.mock('@services/systemService', () => ({
	systemService: {
		getSystemInfo: vi.fn(),
	},
}));

// ── Import after mocks ────────────────────────────────────────────────────────

import { useSettingsContext } from '@contexts/SettingsContext';
import { modelService } from '@services/modelService';
import { settingsService } from '@services/settingsService';
import { systemService } from '@services/systemService';
import { listen } from '@tauri-apps/api/event';
import type { VocylSettings } from '@models/settings';

const mockUseSettingsContext = vi.mocked(useSettingsContext);
const mockModelService = vi.mocked(modelService);
const mockSettingsService = vi.mocked(settingsService);
const mockSystemService = vi.mocked(systemService);
const mockListen = vi.mocked(listen);

function makeContext(overrides = {}) {
	return {
		loading: false,
		exists: false,
		settings: {} as VocylSettings,
		saveSettings: vi.fn().mockResolvedValue(undefined),
		setExists: vi.fn(),
		error: null,
		...overrides,
	} as ReturnType<typeof useSettingsContext>;
}

beforeEach(() => {
	vi.clearAllMocks();
	mockUseSettingsContext.mockReturnValue(makeContext());
	mockModelService.listDownloadedFiles.mockResolvedValue([]);
	mockModelService.isKokoroInstalled.mockResolvedValue(false);
	mockSystemService.getSystemInfo.mockResolvedValue({ total_ram_mb: 8192, cpu_count: 8 });
	mockListen.mockResolvedValue(vi.fn());
	mockSettingsService.updateShortcuts.mockResolvedValue(undefined);
	mockSettingsService.suspendShortcuts.mockResolvedValue(undefined);
	mockSettingsService.resumeShortcuts.mockResolvedValue(undefined);
});

// ── Helper: advance to a specific step ───────────────────────────────────────

async function advanceTo(targetStep: 'tts' | 'stt' | 'keybinds' | 'finish') {
	const user = userEvent.setup();

	// Welcome → TTS
	await user.click(screen.getByRole('button', { name: /get started/i }));
	if (targetStep === 'tts') return;

	// TTS → STT
	await user.click(screen.getByRole('button', { name: /continue/i }));
	if (targetStep === 'stt') return;

	// STT → Keybinds
	await user.click(screen.getByRole('button', { name: /continue/i }));
	if (targetStep === 'keybinds') return;

	// Keybinds → Finish
	await user.click(screen.getByRole('button', { name: /finish setup/i }));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('SetupScreen wizard', () => {
	describe('Welcome step', () => {
		it('renders the welcome heading', () => {
			render(<SetupScreen />);
			expect(screen.getByText('Welcome to Vocyl')).toBeInTheDocument();
		});

		it('has a "Get Started" button', () => {
			render(<SetupScreen />);
			expect(screen.getByRole('button', { name: /get started/i })).toBeInTheDocument();
		});

		it('lists the three setup items', () => {
			render(<SetupScreen />);
			expect(screen.getByText(/text-to-speech voice model/i)).toBeInTheDocument();
			expect(screen.getByText(/speech-to-text transcription model/i)).toBeInTheDocument();
			expect(screen.getByText(/keyboard shortcuts/i)).toBeInTheDocument();
		});
	});

	describe('TTS step', () => {
		it('advances to TTS step on "Get Started"', async () => {
			render(<SetupScreen />);
			await advanceTo('tts');
			expect(screen.getByText('Text-to-Speech Model')).toBeInTheDocument();
		});

		it('shows the Kokoro model row', async () => {
			render(<SetupScreen />);
			await advanceTo('tts');
			expect(screen.getByText('Kokoro English Model')).toBeInTheDocument();
		});

		it('shows a GPL-3 notice', async () => {
			render(<SetupScreen />);
			await advanceTo('tts');
			// Exact string match on the span to avoid matching parent container too
			expect(screen.getByText('Third-party GPL-3.0 software:')).toBeInTheDocument();
		});

		it('has a "Skip for now" link', async () => {
			render(<SetupScreen />);
			await advanceTo('tts');
			expect(screen.getByText(/skip for now/i)).toBeInTheDocument();
		});

		it('"Skip for now" advances to STT step', async () => {
			const user = userEvent.setup();
			render(<SetupScreen />);
			await advanceTo('tts');
			await user.click(screen.getByText(/skip for now/i));
			expect(screen.getByText('Speech-to-Text Model')).toBeInTheDocument();
		});

		it('"Continue" advances to STT step', async () => {
			render(<SetupScreen />);
			await advanceTo('stt');
			expect(screen.getByText('Speech-to-Text Model')).toBeInTheDocument();
		});
	});

	describe('STT step', () => {
		it('shows Whisper model list', async () => {
			render(<SetupScreen />);
			await advanceTo('stt');
			// Use exact strings that only match the label spans (not their parent row divs)
			expect(screen.getByText('Tiny (~75 MB)')).toBeInTheDocument();
			expect(screen.getByText('Base (~142 MB)')).toBeInTheDocument();
		});

		it('auto-selects the first downloaded model when none is selected', async () => {
			// Simulate tiny already being downloaded before the wizard opens
			mockModelService.listDownloadedFiles.mockResolvedValue(['ggml-tiny.bin']);

			render(<SetupScreen />);
			await advanceTo('stt');

			// The tiny model should be shown as active ("Using" badge replaces "Use" button)
			await waitFor(() => {
				expect(screen.getByText('Using')).toBeInTheDocument();
			});
		});

		it('shows "Recommended" badge on the RAM-based suggestion (8 GB → small)', async () => {
			render(<SetupScreen />);
			await waitFor(() => {
				expect(mockSystemService.getSystemInfo).toHaveBeenCalled();
			});
			await advanceTo('stt');
			// 8 GB → 'small' model
			expect(screen.getByText('Recommended')).toBeInTheDocument();
		});

		it('shows RAM info in the description when available', async () => {
			render(<SetupScreen />);
			await advanceTo('stt');
			await waitFor(() => {
				expect(screen.getByText(/8 GB RAM/i)).toBeInTheDocument();
			});
		});

		it('"Skip for now" advances to Keybinds step', async () => {
			const user = userEvent.setup();
			render(<SetupScreen />);
			await advanceTo('stt');
			await user.click(screen.getByText(/skip for now/i));
			expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument();
		});

		it('"Continue" advances to Keybinds step', async () => {
			render(<SetupScreen />);
			await advanceTo('keybinds');
			expect(screen.getByText('Keyboard Shortcuts')).toBeInTheDocument();
		});
	});

	describe('Keybinds step', () => {
		it('shows default shortcuts', async () => {
			render(<SetupScreen />);
			await advanceTo('keybinds');
			// Default shortcuts are Alt+A and Alt+S
			expect(screen.getByText('Alt+A')).toBeInTheDocument();
			expect(screen.getByText('Alt+S')).toBeInTheDocument();
		});

		it('has a "Finish Setup" button', async () => {
			render(<SetupScreen />);
			await advanceTo('keybinds');
			expect(screen.getByRole('button', { name: /finish setup/i })).toBeInTheDocument();
		});

		it('"Finish Setup" advances to Finish step', async () => {
			render(<SetupScreen />);
			await advanceTo('finish');
			expect(screen.getByText(/you're all set/i)).toBeInTheDocument();
		});

		it('"Skip for now" also advances to Finish step', async () => {
			const user = userEvent.setup();
			render(<SetupScreen />);
			await advanceTo('keybinds');
			await user.click(screen.getByText(/skip for now/i));
			expect(screen.getByText(/you're all set/i)).toBeInTheDocument();
		});
	});

	describe('Finish step', () => {
		it('shows the success heading', async () => {
			render(<SetupScreen />);
			await advanceTo('finish');
			expect(screen.getByText(/you're all set/i)).toBeInTheDocument();
		});

		it('shows taskbar tray instructions', async () => {
			render(<SetupScreen />);
			await advanceTo('finish');
			expect(screen.getByText(/system tray icon/i)).toBeInTheDocument();
		});

		it('shows the chosen shortcuts', async () => {
			render(<SetupScreen />);
			await advanceTo('finish');
			// Default shortcuts should appear in the finish screen
			const pttKeys = screen.getAllByText('Alt+A');
			expect(pttKeys.length).toBeGreaterThan(0);
		});

		it('"Open Settings" calls saveSettings', async () => {
			const saveSettings = vi.fn().mockResolvedValue(undefined);
			mockUseSettingsContext.mockReturnValue(makeContext({ saveSettings }));

			const user = userEvent.setup();
			render(<SetupScreen />);
			await advanceTo('finish');
			await user.click(screen.getByRole('button', { name: /open settings/i }));

			await waitFor(() => {
				expect(saveSettings).toHaveBeenCalledOnce();
			});
		});

		it('"Open Settings" calls updateShortcuts', async () => {
			const user = userEvent.setup();
			render(<SetupScreen />);
			await advanceTo('finish');
			await user.click(screen.getByRole('button', { name: /open settings/i }));

			await waitFor(() => {
				expect(mockSettingsService.updateShortcuts).toHaveBeenCalledWith('Alt+A', 'Alt+S');
			});
		});
	});
});
