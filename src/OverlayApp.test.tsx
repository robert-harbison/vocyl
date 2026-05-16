import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, act, waitFor } from '@testing-library/react';
import OverlayApp from './OverlayApp';
import type { OverlayState } from '@components/OverlayIndicator';

vi.mock('@components/OverlayIndicator', () => ({
	default: ({ state }: { state: OverlayState }) => (
		<div data-testid="overlay-indicator" data-state={state}>
			{state}
		</div>
	),
}));

vi.mock('@tauri-apps/api/event');

import { listen } from '@tauri-apps/api/event';

const mockListen = vi.mocked(listen);

beforeEach(() => {
	vi.clearAllMocks();
	mockListen.mockResolvedValue(vi.fn());
});

describe('OverlayApp', () => {
	it('renders OverlayIndicator with the default "listening" state', () => {
		render(<OverlayApp />);
		expect(screen.getByTestId('overlay-indicator')).toHaveTextContent('listening');
	});

	it('registers a listener for the overlay-state event', async () => {
		render(<OverlayApp />);
		await waitFor(() => {
			expect(mockListen).toHaveBeenCalledWith('overlay-state', expect.any(Function));
		});
	});

	it.each<OverlayState>(['waiting', 'processing', 'reading', 'no-text', 'error'])(
		'updates to state "%s" when the overlay-state event fires',
		(newState) => {
			let capturedHandler: ((event: { payload: OverlayState }) => void) | null = null;
			mockListen.mockImplementation((_event, cb) => {
				capturedHandler = cb as (event: { payload: OverlayState }) => void;
				return Promise.resolve(vi.fn());
			});

			render(<OverlayApp />);

			// listen() is called synchronously during the effect, so capturedHandler is set after render.
			act(() => {
				capturedHandler!({ payload: newState });
			});

			expect(screen.getByTestId('overlay-indicator')).toHaveTextContent(newState);
		},
	);

	it('cleans up the event listener on unmount', async () => {
		const mockUnlisten = vi.fn();
		mockListen.mockResolvedValue(mockUnlisten);

		const { unmount } = render(<OverlayApp />);
		unmount();
		await waitFor(() => {
			expect(mockUnlisten).toHaveBeenCalledOnce();
		});
	});
});
