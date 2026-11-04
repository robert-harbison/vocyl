import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ModelRow from './ModelRow';
import { modelService } from '@services/modelService';

vi.mock('@services/modelService', () => ({
	modelService: {
		downloadFile: vi.fn(),
		deleteFile: vi.fn(),
	},
}));

vi.mock('@services/logService');

const mockDownloadFile = vi.mocked(modelService.downloadFile);
const mockDeleteFile = vi.mocked(modelService.deleteFile);

const baseProps = {
	id: 'tiny',
	label: 'Tiny (~75 MB)',
	sizeMb: 75,
	files: [{ filename: 'ggml-tiny.bin', url: 'https://example.com/ggml-tiny.bin' }],
	dir: 'stt' as const,
	downloadedFiles: [],
	dlStates: {},
	activeId: null,
	onUse: vi.fn(),
	onRefresh: vi.fn(),
};

beforeEach(() => {
	vi.clearAllMocks();
});

describe('ModelRow', () => {
	describe('rendering', () => {
		it('renders the label', () => {
			render(<ModelRow {...baseProps} />);
			expect(screen.getByText('Tiny (~75 MB)')).toBeInTheDocument();
		});

		it('renders size in MB when under 1000', () => {
			render(<ModelRow {...baseProps} sizeMb={466} />);
			expect(screen.getByText('466 MB')).toBeInTheDocument();
		});

		it('renders size in GB when 1000 or more', () => {
			render(<ModelRow {...baseProps} sizeMb={1500} />);
			expect(screen.getByText('1.5 GB')).toBeInTheDocument();
		});

		it('does not show "Recommended" badge by default', () => {
			render(<ModelRow {...baseProps} />);
			expect(screen.queryByText('Recommended')).not.toBeInTheDocument();
		});

		it('shows "Recommended" badge when recommended=true', () => {
			render(<ModelRow {...baseProps} recommended />);
			expect(screen.getByText('Recommended')).toBeInTheDocument();
		});
	});

	describe('not downloaded', () => {
		it('shows a Download button', () => {
			render(<ModelRow {...baseProps} downloadedFiles={[]} />);
			expect(screen.getByRole('button', { name: 'Download' })).toBeInTheDocument();
		});

		it('does not show Use or Delete buttons', () => {
			render(<ModelRow {...baseProps} downloadedFiles={[]} />);
			expect(screen.queryByRole('button', { name: 'Use' })).not.toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
		});
	});

	describe('downloaded but not active', () => {
		const downloadedProps = { ...baseProps, downloadedFiles: ['ggml-tiny.bin'] };

		it('shows Use and Delete buttons', () => {
			render(<ModelRow {...downloadedProps} />);
			expect(screen.getByRole('button', { name: 'Use' })).toBeInTheDocument();
			expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
		});

		it('does not show the Download button', () => {
			render(<ModelRow {...downloadedProps} />);
			expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
		});
	});

	describe('downloaded and active', () => {
		const activeProps = {
			...baseProps,
			downloadedFiles: ['ggml-tiny.bin'],
			activeId: 'tiny',
		};

		it('shows "Using" badge instead of Use button', () => {
			render(<ModelRow {...activeProps} />);
			expect(screen.getByText('Using')).toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Use' })).not.toBeInTheDocument();
		});

		it('still shows Delete button', () => {
			render(<ModelRow {...activeProps} />);
			expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
		});
	});

	describe('downloading in progress', () => {
		const dlProps = {
			...baseProps,
			dlStates: { 'stt/ggml-tiny.bin': 42 },
		};

		it('shows the progress percentage', () => {
			render(<ModelRow {...dlProps} />);
			expect(screen.getByText('42%')).toBeInTheDocument();
		});

		it('does not show Download, Use, or Delete buttons', () => {
			render(<ModelRow {...dlProps} />);
			expect(screen.queryByRole('button', { name: 'Download' })).not.toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Use' })).not.toBeInTheDocument();
			expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
		});
	});

	describe('Download button behaviour', () => {
		it('calls modelService.downloadFile with the correct arguments', async () => {
			mockDownloadFile.mockResolvedValueOnce(undefined);
			render(<ModelRow {...baseProps} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			await waitFor(() => {
				expect(mockDownloadFile).toHaveBeenCalledWith(
					'stt',
					'ggml-tiny.bin',
					'https://example.com/ggml-tiny.bin',
				);
			});
		});

		it('calls onRefresh after a successful download', async () => {
			const onRefresh = vi.fn();
			mockDownloadFile.mockResolvedValueOnce(undefined);
			render(<ModelRow {...baseProps} onRefresh={onRefresh} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			await waitFor(() => {
				expect(onRefresh).toHaveBeenCalledOnce();
			});
		});

		it('shows "Queued" badge while the download is pending', async () => {
			// Never resolves so the component stays in the pending state
			mockDownloadFile.mockReturnValueOnce(new Promise(() => undefined));
			render(<ModelRow {...baseProps} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			expect(await screen.findByText('Queued')).toBeInTheDocument();
		});

		it('clears the pending state on download error', async () => {
			mockDownloadFile.mockRejectedValueOnce(new Error('network error'));
			render(<ModelRow {...baseProps} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			// After the error the Download button should be restored
			await waitFor(() => {
				expect(screen.getByRole('button', { name: 'Download' })).toBeInTheDocument();
			});
		});

		it('clears the pending flag when files appear on disk', async () => {
			mockDownloadFile.mockReturnValueOnce(new Promise(() => undefined));
			const { rerender } = render(<ModelRow {...baseProps} />);
			await userEvent.click(screen.getByRole('button', { name: 'Download' }));
			expect(await screen.findByText('Queued')).toBeInTheDocument();

			// Simulate the file appearing on disk
			rerender(<ModelRow {...baseProps} downloadedFiles={['ggml-tiny.bin']} />);
			await waitFor(() => {
				expect(screen.queryByText('Queued')).not.toBeInTheDocument();
			});
		});
	});

	describe('Delete button behaviour', () => {
		const downloadedProps = { ...baseProps, downloadedFiles: ['ggml-tiny.bin'] };

		it('calls modelService.deleteFile with the correct arguments', async () => {
			mockDeleteFile.mockResolvedValueOnce(undefined);
			render(<ModelRow {...downloadedProps} />);
			await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
			await waitFor(() => {
				expect(mockDeleteFile).toHaveBeenCalledWith('stt', 'ggml-tiny.bin');
			});
		});

		it('calls onRefresh after deletion', async () => {
			const onRefresh = vi.fn();
			mockDeleteFile.mockResolvedValueOnce(undefined);
			render(<ModelRow {...downloadedProps} onRefresh={onRefresh} />);
			await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
			await waitFor(() => {
				expect(onRefresh).toHaveBeenCalledOnce();
			});
		});

		it('calls deleteFile for every file in a multi-file model', async () => {
			mockDeleteFile.mockResolvedValue(undefined);
			render(
				<ModelRow
					{...baseProps}
					files={[
						{ filename: 'model.bin', url: 'https://example.com/model.bin' },
						{ filename: 'vocab.bin', url: 'https://example.com/vocab.bin' },
					]}
					downloadedFiles={['model.bin', 'vocab.bin']}
				/>,
			);
			await userEvent.click(screen.getByRole('button', { name: 'Delete' }));
			await waitFor(() => {
				expect(mockDeleteFile).toHaveBeenCalledTimes(2);
				expect(mockDeleteFile).toHaveBeenCalledWith('stt', 'model.bin');
				expect(mockDeleteFile).toHaveBeenCalledWith('stt', 'vocab.bin');
			});
		});
	});

	describe('Use button behaviour', () => {
		it('calls onUse with the row id', async () => {
			const onUse = vi.fn();
			render(<ModelRow {...baseProps} downloadedFiles={['ggml-tiny.bin']} onUse={onUse} />);
			await userEvent.click(screen.getByRole('button', { name: 'Use' }));
			expect(onUse).toHaveBeenCalledWith('tiny');
		});
	});
});
