import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Button from './index';

describe('Button', () => {
	it('renders children', () => {
		render(<Button>Save</Button>);
		expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
	});

	it('shows "Saving..." and is disabled when loading', () => {
		render(<Button loading>Save</Button>);
		const btn = screen.getByRole('button', { name: 'Saving...' });
		expect(btn).toBeDisabled();
	});

	it('is disabled when disabled prop is set', () => {
		render(<Button disabled>Save</Button>);
		expect(screen.getByRole('button')).toBeDisabled();
	});

	it('applies w-full when fullWidth', () => {
		render(<Button fullWidth>Save</Button>);
		expect(screen.getByRole('button').className).toContain('w-full');
	});

	it('does not apply w-full by default', () => {
		render(<Button>Save</Button>);
		expect(screen.getByRole('button').className).not.toContain('w-full');
	});

	it('fires onClick when not disabled', async () => {
		const onClick = vi.fn();
		render(<Button onClick={onClick}>Save</Button>);
		await userEvent.click(screen.getByRole('button'));
		expect(onClick).toHaveBeenCalledOnce();
	});

	it('does not fire onClick when loading', async () => {
		const onClick = vi.fn();
		render(
			<Button loading onClick={onClick}>
				Save
			</Button>,
		);
		await userEvent.click(screen.getByRole('button'));
		expect(onClick).not.toHaveBeenCalled();
	});
});
