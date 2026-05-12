import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import SettingsSection from './index';

describe('SettingsSection', () => {
	it('renders the title', () => {
		render(
			<SettingsSection title="Audio">
				<span />
			</SettingsSection>,
		);
		expect(screen.getByText('Audio')).toBeInTheDocument();
	});

	it('renders children', () => {
		render(
			<SettingsSection title="Audio">
				<p>Child content</p>
			</SettingsSection>,
		);
		expect(screen.getByText('Child content')).toBeInTheDocument();
	});

	it('renders as a section element', () => {
		const { container } = render(
			<SettingsSection title="Audio">
				<span />
			</SettingsSection>,
		);
		expect(container.querySelector('section')).toBeInTheDocument();
	});

	it('renders title as an h2', () => {
		render(
			<SettingsSection title="Audio">
				<span />
			</SettingsSection>,
		);
		expect(screen.getByRole('heading', { level: 2, name: 'Audio' })).toBeInTheDocument();
	});
});
