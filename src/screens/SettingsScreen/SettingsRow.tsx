interface Props {
	label: string;
	description: string;
	children: React.ReactNode;
}

export default function SettingsRow({ label, description, children }: Props) {
	return (
		<div className="flex items-center justify-between px-4 py-3 gap-4">
			<div className="min-w-0">
				<p className="text-sm font-medium text-text-strong">{label}</p>
				<p className="text-xs text-text-faint mt-0.5">{description}</p>
			</div>
			<div className="shrink-0">{children}</div>
		</div>
	);
}
