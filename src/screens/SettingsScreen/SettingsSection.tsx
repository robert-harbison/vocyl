interface Props {
	title: string;
	children: React.ReactNode;
}

export default function SettingsSection({ title, children }: Props) {
	return (
		<section>
			<h2 className="text-xs font-semibold uppercase tracking-widest text-text-faint mb-4">
				{title}
			</h2>
			<div className="bg-bg-surface rounded-xl border border-border divide-y divide-border">
				{children}
			</div>
		</section>
	);
}
