import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import Toast from './index';

describe('Toast', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.runOnlyPendingTimers();
		vi.useRealTimers();
	});

	describe('rendering', () => {
		it('renders the message text', () => {
			render(<Toast message="Something went wrong" onDismiss={vi.fn()} />);
			expect(screen.getByText('Something went wrong')).toBeInTheDocument();
		});

		it('renders the warning icon', () => {
			const { container } = render(<Toast message="Error!" onDismiss={vi.fn()} />);
			// lucide-react renders an <svg> element
			expect(container.querySelector('svg')).toBeInTheDocument();
		});

		it('is visible (opacity-100) on initial render', () => {
			const { container } = render(<Toast message="Hello" onDismiss={vi.fn()} />);
			const toast = container.firstChild as HTMLElement;
			expect(toast).toHaveClass('opacity-100');
			expect(toast).not.toHaveClass('opacity-0');
		});

		it('has the correct positioning classes', () => {
			const { container } = render(<Toast message="Hello" onDismiss={vi.fn()} />);
			const toast = container.firstChild as HTMLElement;
			expect(toast).toHaveClass('fixed');
			expect(toast).toHaveClass('bottom-5');
			expect(toast).toHaveClass('left-1/2');
			expect(toast).toHaveClass('-translate-x-1/2');
		});
	});

	describe('auto-dismiss timer', () => {
		it('becomes invisible (opacity-0) after the default duration of 4000ms', () => {
			const { container } = render(<Toast message="Hello" onDismiss={vi.fn()} />);
			const toast = container.firstChild as HTMLElement;

			expect(toast).toHaveClass('opacity-100');

			act(() => {
				vi.advanceTimersByTime(4000);
			});

			expect(toast).toHaveClass('opacity-0');
			expect(toast).not.toHaveClass('opacity-100');
		});

		it('calls onDismiss 300ms after becoming invisible (after default duration)', () => {
			const onDismiss = vi.fn();
			render(<Toast message="Hello" onDismiss={onDismiss} />);

			act(() => {
				vi.advanceTimersByTime(4000);
			}); // hide animation starts
			expect(onDismiss).not.toHaveBeenCalled();

			act(() => {
				vi.advanceTimersByTime(300);
			}); // fade-out completes
			expect(onDismiss).toHaveBeenCalledOnce();
		});

		it('respects a custom duration prop', () => {
			const onDismiss = vi.fn();
			const { container } = render(
				<Toast message="Hello" duration={1500} onDismiss={onDismiss} />,
			);
			const toast = container.firstChild as HTMLElement;

			act(() => {
				vi.advanceTimersByTime(1499);
			});
			expect(toast).toHaveClass('opacity-100');
			expect(onDismiss).not.toHaveBeenCalled();

			act(() => {
				vi.advanceTimersByTime(1);
			}); // total: 1500ms
			expect(toast).toHaveClass('opacity-0');

			act(() => {
				vi.advanceTimersByTime(300);
			}); // fade-out done
			expect(onDismiss).toHaveBeenCalledOnce();
		});

		it('does not call onDismiss before the combined duration + fade time', () => {
			const onDismiss = vi.fn();
			render(<Toast message="Hello" duration={2000} onDismiss={onDismiss} />);

			// Advance exactly to the end of the duration so React can flush the state
			// update (visible → false) and register the 300 ms fade timer before we
			// continue advancing.
			act(() => {
				vi.advanceTimersByTime(2000);
			}); // duration elapses, fade-out begins

			act(() => {
				vi.advanceTimersByTime(299);
			}); // 299 ms into the fade timer
			expect(onDismiss).not.toHaveBeenCalled();

			act(() => {
				vi.advanceTimersByTime(1);
			}); // fade timer completes
			expect(onDismiss).toHaveBeenCalledOnce();
		});
	});

	describe('cleanup on unmount', () => {
		it('does not call onDismiss if unmounted before the duration elapses', () => {
			const onDismiss = vi.fn();
			const { unmount } = render(
				<Toast message="Hello" duration={4000} onDismiss={onDismiss} />,
			);

			act(() => {
				vi.advanceTimersByTime(2000);
			});
			unmount();
			act(() => {
				vi.advanceTimersByTime(5000);
			}); // well past full cycle

			expect(onDismiss).not.toHaveBeenCalled();
		});

		it('does not call onDismiss if unmounted during the fade-out window', () => {
			const onDismiss = vi.fn();
			const { unmount } = render(
				<Toast message="Hello" duration={4000} onDismiss={onDismiss} />,
			);

			act(() => {
				vi.advanceTimersByTime(4000);
			}); // triggers fade-out
			unmount();
			act(() => {
				vi.advanceTimersByTime(300);
			}); // fade timer fires (but component is gone)

			// onDismiss may or may not be called depending on cleanup ordering;
			// the important thing is no unhandled errors are thrown
			// In React strict teardown the effect cleanup runs, so it should NOT be called.
			expect(onDismiss).not.toHaveBeenCalled();
		});
	});

	describe('transition classes', () => {
		it('always has the transition-opacity class for smooth animation', () => {
			const { container } = render(<Toast message="Hello" onDismiss={vi.fn()} />);
			expect(container.firstChild).toHaveClass('transition-opacity');
		});

		it('always has the duration-300 class for the CSS transition speed', () => {
			const { container } = render(<Toast message="Hello" onDismiss={vi.fn()} />);
			expect(container.firstChild).toHaveClass('duration-300');
		});
	});
});
