import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { JSX } from 'react';
import ErrorBoundary from './index';
import { logger } from '@services/logService';

vi.mock('@services/logService');

// Vitest surfaces React's console.error calls for every caught boundary error.
// Suppress them so the test output stays clean.
// Also clear all mock call counts so logger.error tallies don't bleed between tests.
beforeEach(() => {
	vi.clearAllMocks();
	vi.spyOn(console, 'error').mockImplementation(vi.fn());
});

function GoodChild(): JSX.Element {
	return <p>All good</p>;
}

function BadChild({ message = 'boom' }: { message?: string }): JSX.Element {
	throw new Error(message);
}

function NonErrorChild(): JSX.Element {
	// eslint-disable-next-line @typescript-eslint/only-throw-error
	throw 'raw string error';
}

describe('ErrorBoundary', () => {
	describe('happy path', () => {
		it('renders children when there is no error', () => {
			render(
				<ErrorBoundary>
					<GoodChild />
				</ErrorBoundary>,
			);
			expect(screen.getByText('All good')).toBeInTheDocument();
		});

		it('does not render the error UI when children render successfully', () => {
			render(
				<ErrorBoundary>
					<GoodChild />
				</ErrorBoundary>,
			);
			expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
		});
	});

	describe('error UI', () => {
		it('renders the "Something went wrong" heading when a child throws', () => {
			render(
				<ErrorBoundary>
					<BadChild />
				</ErrorBoundary>,
			);
			expect(screen.getByText('Something went wrong')).toBeInTheDocument();
		});

		it('renders the error message from a thrown Error object', () => {
			render(
				<ErrorBoundary>
					<BadChild message="disk full" />
				</ErrorBoundary>,
			);
			expect(screen.getByText('disk full')).toBeInTheDocument();
		});

		it('renders the stringified value when a non-Error is thrown', () => {
			render(
				<ErrorBoundary>
					<NonErrorChild />
				</ErrorBoundary>,
			);
			expect(screen.getByText('raw string error')).toBeInTheDocument();
		});

		it('renders the "Check the log file for details" hint', () => {
			render(
				<ErrorBoundary>
					<BadChild />
				</ErrorBoundary>,
			);
			expect(screen.getByText('Check the log file for details.')).toBeInTheDocument();
		});

		it('does not render children after an error is caught', () => {
			render(
				<ErrorBoundary>
					<BadChild />
					<p>Should not appear</p>
				</ErrorBoundary>,
			);
			expect(screen.queryByText('Should not appear')).not.toBeInTheDocument();
		});
	});

	describe('logging', () => {
		it('calls logger.error when a child throws an Error', () => {
			render(
				<ErrorBoundary>
					<BadChild message="oops" />
				</ErrorBoundary>,
			);
			expect(logger.error).toHaveBeenCalledOnce();
			expect(logger.error).toHaveBeenCalledWith(expect.stringContaining('Error: oops'));
		});

		it('includes the default context "App" in the log message when context is omitted', () => {
			render(
				<ErrorBoundary>
					<BadChild />
				</ErrorBoundary>,
			);
			expect(logger.error).toHaveBeenCalledWith(
				expect.stringContaining('[ErrorBoundary:App]'),
			);
		});

		it('includes the custom context label in the log message when context is provided', () => {
			render(
				<ErrorBoundary context="Overlay">
					<BadChild />
				</ErrorBoundary>,
			);
			expect(logger.error).toHaveBeenCalledWith(
				expect.stringContaining('[ErrorBoundary:Overlay]'),
			);
		});

		it('includes the component stack in the log message', () => {
			render(
				<ErrorBoundary>
					<BadChild />
				</ErrorBoundary>,
			);
			expect(logger.error).toHaveBeenCalledWith(expect.stringContaining('Component stack:'));
		});

		it('calls logger.error with the stringified value when a non-Error is thrown', () => {
			render(
				<ErrorBoundary>
					<NonErrorChild />
				</ErrorBoundary>,
			);
			expect(logger.error).toHaveBeenCalledWith(expect.stringContaining('raw string error'));
		});
	});
});
