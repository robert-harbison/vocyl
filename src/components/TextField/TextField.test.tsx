import { describe, it, expect } from 'vitest';
import { createRef } from 'react';
import { render, screen } from '@testing-library/react';
import TextField from './index';

describe('TextField', () => {
	it('renders the input', () => {
		render(<TextField />);
		expect(screen.getByRole('textbox')).toBeInTheDocument();
	});

	it('renders the label when provided', () => {
		render(<TextField label="Test Property" id="test" />);
		expect(screen.getByLabelText('Test Property')).toBeInTheDocument();
	});

	it('does not render a label when not provided', () => {
		render(<TextField />);
		expect(screen.queryByRole('label')).not.toBeInTheDocument();
	});

	it('renders the hint when provided and no error', () => {
		render(<TextField hint="This is a hint" />);
		expect(screen.getByText('This is a hint')).toBeInTheDocument();
	});

	it('renders the error and hides hint when error is provided', () => {
		render(<TextField hint="This is a hint" error="This field is required" />);
		expect(screen.getByText('This field is required')).toBeInTheDocument();
		expect(screen.queryByText('This is a hint')).not.toBeInTheDocument();
	});

	it('applies error border class when error is set', () => {
		render(<TextField error="Oops" />);
		expect(screen.getByRole('textbox').className).toContain('border-error');
	});

	it('applies normal border class when no error', () => {
		render(<TextField />);
		expect(screen.getByRole('textbox').className).toContain('border-border-input');
	});

	it('passes through additional className', () => {
		render(<TextField className="w-56" />);
		expect(screen.getByRole('textbox').className).toContain('w-56');
	});

	it('forwards the ref to the input element', () => {
		const ref = createRef<HTMLInputElement>();
		render(<TextField ref={ref} />);
		expect(ref.current).toBe(screen.getByRole('textbox'));
	});

	it('passes through input attributes', () => {
		render(<TextField placeholder="Enter a value..." type="text" />);
		expect(screen.getByPlaceholderText('Enter a value...')).toBeInTheDocument();
	});
});
