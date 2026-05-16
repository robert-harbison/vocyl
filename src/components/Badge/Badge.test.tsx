import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import Badge from './index';

describe('Badge', () => {
	describe('children', () => {
		it('renders text children', () => {
			render(<Badge>Hello</Badge>);
			expect(screen.getByText('Hello')).toBeInTheDocument();
		});

		it('renders complex node children', () => {
			render(
				<Badge>
					<span data-testid="child-node">Label</span>
				</Badge>,
			);
			expect(screen.getByTestId('child-node')).toBeInTheDocument();
		});
	});

	describe('icon', () => {
		it('renders the icon when provided', () => {
			render(<Badge icon={<svg data-testid="badge-icon" />}>Text</Badge>);
			expect(screen.getByTestId('badge-icon')).toBeInTheDocument();
		});

		it('does not render an icon placeholder when icon is omitted', () => {
			const { container } = render(<Badge>Text</Badge>);
			// The span should only contain the children text, no extra element
			expect(container.querySelector('svg')).not.toBeInTheDocument();
		});

		it('renders icon before children', () => {
			const { container } = render(
				<Badge icon={<svg data-testid="badge-icon" />}>Label</Badge>,
			);
			const span = container.querySelector('span')!;
			const nodes = Array.from(span.childNodes).filter(
				(n) => n.nodeType !== Node.TEXT_NODE || n.textContent?.trim(),
			);
			// First child should be the icon SVG
			expect((nodes[0] as Element).tagName).toBe('svg');
		});
	});

	describe('variant classes', () => {
		it('applies neutral classes by default', () => {
			const { container } = render(<Badge>Default</Badge>);
			const span = container.querySelector('span')!;
			expect(span).toHaveClass('bg-bg-elevated');
			expect(span).toHaveClass('text-text-faint');
		});

		it('applies neutral classes when variant is explicitly neutral', () => {
			const { container } = render(<Badge variant="neutral">Neutral</Badge>);
			const span = container.querySelector('span')!;
			expect(span).toHaveClass('bg-bg-elevated');
			expect(span).toHaveClass('text-text-faint');
		});

		it('applies primary classes for the primary variant', () => {
			const { container } = render(<Badge variant="primary">Primary</Badge>);
			const span = container.querySelector('span')!;
			expect(span).toHaveClass('bg-primary/10');
			expect(span).toHaveClass('text-primary');
		});

		it('applies success classes for the success variant', () => {
			const { container } = render(<Badge variant="success">Success</Badge>);
			const span = container.querySelector('span')!;
			expect(span).toHaveClass('bg-success/10');
			expect(span).toHaveClass('text-success');
		});

		it('applies error classes for the error variant', () => {
			const { container } = render(<Badge variant="error">Error</Badge>);
			const span = container.querySelector('span')!;
			expect(span).toHaveClass('bg-error/10');
			expect(span).toHaveClass('text-error');
		});

		it('does not bleed classes across variants', () => {
			const { container } = render(<Badge variant="success">Success</Badge>);
			const span = container.querySelector('span')!;
			expect(span).not.toHaveClass('text-primary');
			expect(span).not.toHaveClass('text-error');
			expect(span).not.toHaveClass('text-text-faint');
		});
	});

	describe('base styles', () => {
		it('always applies shared layout classes regardless of variant', () => {
			const variants = ['primary', 'success', 'error', 'neutral'] as const;
			variants.forEach((variant) => {
				const { container } = render(<Badge variant={variant}>Badge</Badge>);
				const span = container.querySelector('span')!;
				expect(span).toHaveClass('inline-flex');
				expect(span).toHaveClass('items-center');
				expect(span).toHaveClass('text-xs');
				expect(span).toHaveClass('font-medium');
				expect(span).toHaveClass('rounded');
			});
		});

		it('renders as a span element', () => {
			const { container } = render(<Badge>Badge</Badge>);
			expect(container.firstChild?.nodeName).toBe('SPAN');
		});
	});
});
