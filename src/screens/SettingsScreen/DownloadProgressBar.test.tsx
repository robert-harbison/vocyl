import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import DownloadProgressBar from './DownloadProgressBar';
describe('DownloadProgressBar', () => {
	it('renders the percentage label', () => {
		render(<DownloadProgressBar progress={42} />);
		expect(screen.getByText('42%')).toBeInTheDocument();
	});
	it('renders 0% at the start', () => {
		render(<DownloadProgressBar progress={0} />);
		expect(screen.getByText('0%')).toBeInTheDocument();
	});
	it('renders 100% when complete', () => {
		render(<DownloadProgressBar progress={100} />);
		expect(screen.getByText('100%')).toBeInTheDocument();
	});
	it('sets the fill bar width to match the progress', () => {
		const { container } = render(<DownloadProgressBar progress={65} />);
		const fill = container.querySelector<HTMLElement>('.bg-primary.rounded-full');
		expect(fill?.style.width).toBe('65%');
	});
});
