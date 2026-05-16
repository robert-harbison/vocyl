import React from 'react';

interface SettingsRowProps {
	label: string;
	description: string;
	children: React.ReactNode;
	error?: string;
}

export default function SettingsRow(props: SettingsRowProps) {
	const { label, description, children, error } = props;

	return (
		<div className="flex items-start justify-between px-4 py-3 gap-4">
			<div className="min-w-0">
				<p className="text-sm font-medium text-text-strong">{label}</p>
				<p className="text-xs text-text-faint mt-0.5">{description}</p>
				{error && <p className="text-xs text-error mt-1">{error}</p>}
			</div>
			<div className="shrink-0">{children}</div>
		</div>
	);
}
