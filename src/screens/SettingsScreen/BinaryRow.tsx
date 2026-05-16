import { useEffect, useRef, type JSX } from 'react';
import { Check, Loader2, Package } from 'lucide-react';

import Badge from '@components/Badge';
import Button from '@components/Button';
import { logger } from '@services/logService';
import type { DownloadStates } from './ModelRow';
import { useDownloadPending } from './useDownloadPending';
import DownloadProgressBar from './DownloadProgressBar';

export interface BinaryRowProps {
	label: string;
	description: string;
	isInstalled: boolean;
	dlKey: string;
	dlStates: DownloadStates;
	onDownload: () => Promise<void>;
	onDelete: () => Promise<void>;
}

export default function BinaryRow({
	label,
	description,
	isInstalled,
	dlKey,
	dlStates,
	onDownload,
	onDelete,
}: BinaryRowProps): JSX.Element {
	const downloading = dlStates[dlKey];
	const isDownloading = typeof downloading === 'number';
	const progress = isDownloading ? downloading : 0;

	const [isPending, setIsPending] = useDownloadPending(isInstalled);

	const hasStartedDownloading = useRef(false);
	useEffect(() => {
		if (isDownloading) hasStartedDownloading.current = true;
		if (!isPending) hasStartedDownloading.current = false;
	}, [isDownloading, isPending]);
	const isUnpacking = isPending && !isDownloading && hasStartedDownloading.current;

	const handleDownload = async () => {
		setIsPending(true);
		try {
			await onDownload();
		} catch (err) {
			logger.info(`BinaryRow: download failed for ${dlKey}: ${String(err)}`);
			setIsPending(false);
		}
	};

	const handleDelete = async () => {
		try {
			await onDelete();
		} catch (err) {
			logger.info(`BinaryRow: delete failed for ${dlKey}: ${String(err)}`);
		}
	};

	return (
		<div className="flex items-center gap-3 px-4 py-3">
			<div className="flex-1 min-w-0">
				<p className="text-sm font-medium text-text-strong">{label}</p>
				<p className="text-xs text-text-faint mt-0.5">{description}</p>
			</div>
			{isInstalled ?
				<div className="flex items-center gap-1.5">
					<Badge variant="success" icon={<Check className="w-3.5 h-3.5" />}>
						Installed
					</Badge>
					<Button
						type="button"
						variant="ghost-destructive"
						size="sm"
						onClick={() => void handleDelete()}
					>
						Delete
					</Button>
				</div>
			: isDownloading ?
				<DownloadProgressBar progress={progress} />
			: isUnpacking ?
				<Badge variant="neutral" icon={<Package className="w-3.5 h-3.5" />}>
					Unpacking
				</Badge>
			: isPending ?
				<Badge variant="neutral" icon={<Loader2 className="w-3.5 h-3.5 animate-spin" />}>
					Queued
				</Badge>
			:	<Button
					type="button"
					variant="secondary"
					size="sm"
					onClick={() => void handleDownload()}
				>
					Download
				</Button>
			}
		</div>
	);
}
