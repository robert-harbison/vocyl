import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Button from './index';

describe('Button', () => {
	describe('children / loading text', () => {
		it('renders children when not loading', () => {
			render(<Button>Save</Button>);
			expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
		});

		it('shows "Saving..." and is disabled when loading', () => {
			render(<Button loading>Save</Button>);
			const btn = screen.getByRole('button', { name: 'Saving...' });
			expect(btn).toBeDisabled();
		});

		it('shows children again once loading is false', () => {
			const { rerender } = render(<Button loading>Save</Button>);
			expect(screen.getByRole('button', { name: 'Saving...' })).toBeInTheDocument();
			rerender(<Button loading={false}>Save</Button>);
			expect(screen.getByRole('button', { name: 'Save' })).toBeInTheDocument();
		});
	});

	describe('disabled', () => {
		it('is disabled when disabled prop is set', () => {
			render(<Button disabled>Save</Button>);
			expect(screen.getByRole('button')).toBeDisabled();
		});

		it('does not fire onClick when disabled', async () => {
			const onClick = vi.fn();
			render(
				<Button disabled onClick={onClick}>
					Save
				</Button>,
			);
			await userEvent.click(screen.getByRole('button'));
			expect(onClick).not.toHaveBeenCalled();
		});
	});

	describe('fullWidth', () => {
		it('applies w-full when fullWidth', () => {
			render(<Button fullWidth>Save</Button>);
			expect(screen.getByRole('button')).toHaveClass('w-full');
		});

		it('does not apply w-full by default', () => {
			render(<Button>Save</Button>);
			expect(screen.getByRole('button')).not.toHaveClass('w-full');
		});
	});

	describe('variants', () => {
		it('applies primary classes by default', () => {
			render(<Button>Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('bg-primary');
			expect(btn).toHaveClass('text-white');
		});

		it('applies secondary classes', () => {
			render(<Button variant="secondary">Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('border');
			expect(btn).toHaveClass('text-primary');
			expect(btn).not.toHaveClass('bg-primary');
		});

		it('applies ghost classes', () => {
			render(<Button variant="ghost">Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('text-text-faint');
			expect(btn).not.toHaveClass('bg-primary');
		});

		it('applies destructive classes', () => {
			render(<Button variant="destructive">Delete</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('text-error');
			expect(btn).toHaveClass('border');
			expect(btn).not.toHaveClass('bg-primary');
		});

		it('applies ghost-destructive classes', () => {
			render(<Button variant="ghost-destructive">Delete</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('text-error/70');
			expect(btn).not.toHaveClass('border');
			expect(btn).not.toHaveClass('bg-primary');
		});
	});

	describe('sizes', () => {
		it('applies md size classes by default', () => {
			render(<Button>Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('text-sm');
			expect(btn).toHaveClass('rounded-lg');
		});

		it('applies sm size classes', () => {
			render(<Button size="sm">Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('text-xs');
			expect(btn).toHaveClass('rounded');
			expect(btn).not.toHaveClass('rounded-lg');
		});
	});

	describe('className', () => {
		it('merges a custom className with the base classes', () => {
			render(<Button className="my-custom-class">Save</Button>);
			const btn = screen.getByRole('button');
			expect(btn).toHaveClass('my-custom-class');
			expect(btn).toHaveClass('bg-primary'); // base classes still present
		});
	});

	describe('click behaviour', () => {
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

	describe('base styles', () => {
		it('always applies shared layout and focus classes regardless of variant or size', () => {
			const variants = [
				'primary',
				'secondary',
				'ghost',
				'destructive',
				'ghost-destructive',
			] as const;
			variants.forEach((variant) => {
				const { unmount } = render(<Button variant={variant}>Btn</Button>);
				const btn = screen.getByRole('button');
				expect(btn).toHaveClass('inline-flex');
				expect(btn).toHaveClass('items-center');
				expect(btn).toHaveClass('font-semibold');
				expect(btn).toHaveClass('transition-colors');
				expect(btn).toHaveClass('cursor-pointer');
				unmount();
			});
		});
	});
});
