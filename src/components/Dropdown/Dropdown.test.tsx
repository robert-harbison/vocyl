import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Dropdown from './index';
import type { DropdownOption } from './index';

const OPTIONS: DropdownOption[] = [
	{ value: 'apple', label: 'Apple' },
	{ value: 'banana', label: 'Banana' },
	{ value: 'cherry', label: 'Cherry' },
];

function renderDropdown(overrides: Partial<Parameters<typeof Dropdown>[0]> = {}) {
	const onChange = vi.fn();
	const utils = render(
		<Dropdown options={OPTIONS} value={null} onChange={onChange} {...overrides} />,
	);
	return { ...utils, onChange };
}

function getTrigger() {
	// The first button is always the trigger
	return screen.getAllByRole('button')[0];
}

describe('Dropdown', () => {
	describe('initial render', () => {
		it('shows the placeholder when value is null', () => {
			renderDropdown({ placeholder: 'Pick one' });
			expect(screen.getByText('Pick one')).toBeInTheDocument();
		});

		it('uses "Select..." as the default placeholder', () => {
			renderDropdown();
			expect(screen.getByText('Select...')).toBeInTheDocument();
		});

		it('shows the label of the matched option when a value is set', () => {
			renderDropdown({ value: 'banana' });
			expect(screen.getByText('Banana')).toBeInTheDocument();
		});

		it('does not show the options list on first render', () => {
			renderDropdown();
			expect(screen.queryByText('Apple')).not.toBeInTheDocument();
		});

		it('applies a custom className to the wrapper', () => {
			const { container } = renderDropdown({ className: 'custom-wrapper' });
			expect(container.firstChild).toHaveClass('custom-wrapper');
		});
	});

	describe('open / close', () => {
		it('opens the options list when the trigger is clicked', async () => {
			renderDropdown();
			await userEvent.click(getTrigger());
			expect(screen.getByText('Apple')).toBeInTheDocument();
			expect(screen.getByText('Banana')).toBeInTheDocument();
			expect(screen.getByText('Cherry')).toBeInTheDocument();
		});

		it('closes the options list when the trigger is clicked a second time', async () => {
			renderDropdown();
			await userEvent.click(getTrigger());
			await userEvent.click(getTrigger());
			expect(screen.queryByText('Apple')).not.toBeInTheDocument();
		});

		it('rotates the chevron when open', async () => {
			const { container } = renderDropdown();
			const svg = container.querySelector('svg')!;
			expect(svg).not.toHaveClass('rotate-180');
			await userEvent.click(getTrigger());
			expect(svg).toHaveClass('rotate-180');
		});

		it('resets the chevron rotation when closed again', async () => {
			const { container } = renderDropdown();
			const svg = container.querySelector('svg')!;
			await userEvent.click(getTrigger());
			await userEvent.click(getTrigger());
			expect(svg).not.toHaveClass('rotate-180');
		});
	});

	describe('click outside to close', () => {
		it('closes the dropdown when clicking outside the component', async () => {
			renderDropdown();
			await userEvent.click(getTrigger());
			expect(screen.getByText('Apple')).toBeInTheDocument();

			// fire a mousedown event on document body (outside the component)
			await userEvent.click(document.body);
			expect(screen.queryByText('Apple')).not.toBeInTheDocument();
		});

		it('removes the mousedown listener on unmount', () => {
			const removeSpy = vi.spyOn(document, 'removeEventListener');
			const { unmount } = renderDropdown();
			unmount();
			expect(removeSpy).toHaveBeenCalledWith('mousedown', expect.any(Function));
			removeSpy.mockRestore();
		});
	});

	describe('selecting an option', () => {
		it('calls onChange with the option value when an option is clicked', async () => {
			const { onChange } = renderDropdown();
			await userEvent.click(getTrigger());
			await userEvent.click(screen.getByText('Banana'));
			expect(onChange).toHaveBeenCalledOnce();
			expect(onChange).toHaveBeenCalledWith('banana');
		});

		it('closes the list after an option is selected', async () => {
			renderDropdown();
			await userEvent.click(getTrigger());
			await userEvent.click(screen.getByText('Apple'));
			expect(screen.queryByText('Apple')).not.toBeInTheDocument();
		});

		it('does not call onChange when the list is not open', () => {
			const { onChange } = renderDropdown();
			expect(onChange).not.toHaveBeenCalled();
		});
	});

	describe('selected option highlighting', () => {
		it('applies selected classes to the currently selected option', async () => {
			renderDropdown({ value: 'cherry' });
			await userEvent.click(getTrigger());
			// When "cherry" is selected and the list is open, both the trigger button
			// and the "Cherry" option button carry the accessible name "Cherry".
			// The option button is the last one rendered, so we take the last match.
			const cherryBtns = screen.getAllByRole('button', { name: 'Cherry' });
			const cherryOptionBtn = cherryBtns[cherryBtns.length - 1];
			expect(cherryOptionBtn).toHaveClass('bg-primary/10');
			expect(cherryOptionBtn).toHaveClass('text-primary');
		});

		it('does not apply selected classes to unselected options', async () => {
			renderDropdown({ value: 'cherry' });
			await userEvent.click(getTrigger());
			const appleBtn = screen.getByRole('button', { name: 'Apple' });
			expect(appleBtn).not.toHaveClass('bg-primary/10');
			expect(appleBtn).toHaveClass('text-text');
		});
	});

	describe('empty options', () => {
		it('shows "No options available" when the options array is empty', async () => {
			renderDropdown({ options: [] });
			await userEvent.click(getTrigger());
			expect(screen.getByText('No options available')).toBeInTheDocument();
		});

		it('does not render any option buttons when the list is empty', async () => {
			renderDropdown({ options: [] });
			await userEvent.click(getTrigger());
			// Only the trigger button should exist
			expect(screen.getAllByRole('button')).toHaveLength(1);
		});
	});

	describe('disabled state', () => {
		it('disables the trigger button', () => {
			renderDropdown({ disabled: true });
			expect(getTrigger()).toBeDisabled();
		});

		it('does not open the options list when clicking a disabled trigger', async () => {
			renderDropdown({ disabled: true });
			await userEvent.click(getTrigger());
			expect(screen.queryByText('Apple')).not.toBeInTheDocument();
		});

		it('applies disabled styling classes to the trigger', () => {
			renderDropdown({ disabled: true });
			expect(getTrigger()).toHaveClass('cursor-not-allowed');
			expect(getTrigger()).toHaveClass('opacity-50');
		});

		it('does not call onChange when disabled', async () => {
			const { onChange } = renderDropdown({ disabled: true });
			await userEvent.click(getTrigger());
			expect(onChange).not.toHaveBeenCalled();
		});
	});
});
