import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import SettingsRow from './index';

describe('SettingsRow', () => {
	it('renders the label', () => {
		render(
			<SettingsRow label="Volume" description="Set the volume level">
				<span />
			</SettingsRow>,
		);
		expect(screen.getByText('Volume')).toBeInTheDocument();
	});

	it('renders the description', () => {
		render(
			<SettingsRow label="Volume" description="Set the volume level">
				<span />
			</SettingsRow>,
		);
		expect(screen.getByText('Set the volume level')).toBeInTheDocument();
	});

	it('renders children', () => {
		render(
			<SettingsRow label="Volume" description="Set the volume level">
				<input type="range" aria-label="volume slider" />
			</SettingsRow>,
		);
		expect(screen.getByRole('slider', { name: 'volume slider' })).toBeInTheDocument();
	});
});
