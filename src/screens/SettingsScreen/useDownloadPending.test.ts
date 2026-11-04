import { describe, it, expect } from 'vitest';
import { act } from '@testing-library/react';
import { renderHook } from '@testing-library/react';
import { useDownloadPending } from './useDownloadPending';

describe('useDownloadPending', () => {
	it('starts with isPending = false', () => {
		const { result } = renderHook(() => useDownloadPending(false));
		expect(result.current[0]).toBe(false);
	});

	it('allows setting isPending to true via the setter', () => {
		const { result } = renderHook(() => useDownloadPending(false));
		act(() => {
			result.current[1](true);
		});
		expect(result.current[0]).toBe(true);
	});

	it('clears isPending automatically when isComplete flips to true', () => {
		const { result, rerender } = renderHook(
			({ isComplete }) => useDownloadPending(isComplete),
			{ initialProps: { isComplete: false } },
		);

		act(() => {
			result.current[1](true);
		});
		expect(result.current[0]).toBe(true);

		rerender({ isComplete: true });
		expect(result.current[0]).toBe(false);
	});

	it('does not auto-clear isPending if isComplete never changes after being set to true', () => {
		// The effect only re-runs when isComplete changes - if it was already true on mount
		// and never changes again, a subsequent setIsPending(true) won't be cleared.
		// This situation cannot occur in practice (you wouldn't start a download for
		// something already complete), but documents the hook's behavioural boundary.
		const { result } = renderHook(() => useDownloadPending(true));
		act(() => {
			result.current[1](true);
		});
		// isComplete hasn't changed, so the effect won't re-fire.
		expect(result.current[0]).toBe(true);
	});

	it('stays cleared if isComplete remains true', () => {
		const { result, rerender } = renderHook(
			({ isComplete }) => useDownloadPending(isComplete),
			{ initialProps: { isComplete: true } },
		);
		rerender({ isComplete: true });
		expect(result.current[0]).toBe(false);
	});
});
