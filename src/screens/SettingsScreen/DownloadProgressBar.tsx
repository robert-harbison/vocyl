import type { JSX } from 'react';

interface DownloadProgressBarProps {
	progress: number;
}

export default function DownloadProgressBar({ progress }: DownloadProgressBarProps): JSX.Element {
	return (
		<div className="flex items-center gap-2 min-w-28">
			<div className="flex-1 h-1 bg-bg-elevated rounded-full overflow-hidden">
				<div
					className="h-full bg-primary rounded-full transition-all duration-300"
					style={{ width: `${String(progress)}%` }}
				/>
			</div>
			<span className="text-xs text-text-faint w-8 text-right">{progress}%</span>
		</div>
	);
}
