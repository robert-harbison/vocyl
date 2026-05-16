import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import BinaryRow from './BinaryRow';

vi.mock('@services/logService');

const baseProps = {
	label: 'Kokoro English Model',
	description: 'Voice model + phoneme data (~90 MB).',
	isInstalled: false,
	dlKey: 'tts/kokoro',
	dlStates: {},
	onDownload: vi.fn(),
	onDelete: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('BinaryRow', () => {
	describe('rendering', () => {
		it('renders the label', () => {
			render(<BinaryRow {...baseProps} />);
			expect(screen.getByText('Kokoro English Model')).toBeInTheDocument();
		});

		it('renders the description', () => {
			render(<BinaryRow {...baseProps} />);
			expect(screen.getByText('Voice model + phoneme data (~90 MB).')).toBeInTheDocument();
		});
	});

	describe('not installed', () => {
		it('shows a Download button', () => {
			render(<BinaryRow {...baseProps} />);
			expect(screen.getByRole('button', { name: 'Download' })).toBeInTheDocument();
		});

		it('does not show an Installed badge or Delete button', () => {
			render(<BinaryRow {...baseProps} />);
			expect(screen.queryByText('Installed')).not.toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
		});
	});

	describe('installed', () => {
		const installedProps = { ...baseProps, isInstalled: true };

		it('shows an "Installed" badge', () => {
			render(<BinaryRow {...installedProps} />);
			expect(screen.getByText('Installed')).toBeInTheDocument();
		});

		it('shows a Delete button', () => {
			render(<BinaryRow {...installedProps} />);
			expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
		});

		it('does not show a Download button', () => {
			render(<BinaryRow {...installedProps} />);
			expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
		});
	});

	describe('downloading in progress', () => {
		const dlProps = { ...baseProps, dlStates: { 'tts/kokoro': 65 } };

		it('shows the progress percentage', () => {
			render(<BinaryRow {...dlProps} />);
			expect(screen.getByText('65%')).toBeInTheDocument();
		});

		it('does not show Download or Delete buttons', () => {
			render(<BinaryRow {...dlProps} />);
			expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
		});
	});

	describe('Download button behaviour', () => {
		it('calls onDownload when clicked', async () => {
			const onDownload = vi.fn().mockResolvedValueOnce(undefined);
			render(<BinaryRow {...baseProps} onDownload={onDownload} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			await waitFor(() => {
				expect(onDownload).toHaveBeenCalledOnce();
			});
		});

		it('shows "Queued" badge while pending', async () => {
			const onDownload = vi.fn().mockReturnValueOnce(new Promise(() => undefined));
			render(<BinaryRow {...baseProps} onDownload={onDownload} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			expect(await screen.findByText('Queued')).toBeInTheDocument();
		});

		it('clears pending state on download error', async () => {
			const onDownload = vi.fn().mockRejectedValueOnce(new Error('network error'));
			render(<BinaryRow {...baseProps} onDownload={onDownload} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			await waitFor(() => {
				expect(screen.getByRole('button', { name: 'Download' })).toBeInTheDocument();
			});
		});

		it('clears pending flag when isInstalled becomes true', async () => {
			const onDownload = vi.fn().mockReturnValueOnce(new Promise(() => undefined));
			const { rerender } = render(<BinaryRow {...baseProps} onDownload={onDownload} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			expect(await screen.findByText('Queued')).toBeInTheDocument();

			rerender(<BinaryRow {...baseProps} isInstalled onDownload={onDownload} />);
			await waitFor(() => {
				expect(screen.queryByText('Queued')).not.toBeInTheDocument();
				expect(screen.getByText('Installed')).toBeInTheDocument();
			});
		});
	});

	describe('Delete button behaviour', () => {
		it('calls onDelete when clicked', async () => {
			const onDelete = vi.fn().mockResolvedValueOnce(undefined);
			render(<BinaryRow {...baseProps} isInstalled onDelete={onDelete} />);
			await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
			await waitFor(() => {
				expect(onDelete).toHaveBeenCalledOnce();
			});
		});
	});
});
