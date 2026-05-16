import { useCallback, useEffect, useRef, type JSX } from 'react';
import { clsx } from 'clsx';
import { Check, Loader2, Package, Star } from 'lucide-react';

import Badge from '@components/Badge';
import Button from '@components/Button';
import { modelService } from '@services/modelService';
import { logger } from '@services/logService';
import type { ModelFile } from '@/constants/models';
import { useDownloadPending } from './useDownloadPending';
import DownloadProgressBar from './DownloadProgressBar';

// Key = `${dir}/${filename}`, value = 0-100 progress
export type DownloadStates = Record<string, number | 'done'>;

export interface ModelRowProps {
	id: string;
	label: string;
	sizeMb: number;
	files: ModelFile[];
	dir: 'tts' | 'stt';
	downloadedFiles: string[];
	dlStates: DownloadStates;
	activeId: string | null;
	onUse: (id: string) => void;
	onRefresh: () => void;
	/** When true, shows a "Recommended" badge next to the label. */
	recommended?: boolean;
}

export default function ModelRow({
	id,
	label,
	sizeMb,
	files,
	dir,
	downloadedFiles,
	dlStates,
	activeId,
	onUse,
	onRefresh,
	recommended = false,
}: ModelRowProps): JSX.Element {
	const isDownloaded = files.every((f) => downloadedFiles.includes(f.filename));
	const isActive = activeId === id;

	const activeFile = files.find((f) => {
		const key = `${dir}/${f.filename}`;
		return typeof dlStates[key] === 'number';
	});
	const isDownloading = activeFile !== undefined;
	const progress = activeFile ? (dlStates[`${dir}/${activeFile.filename}`] as number) : 0;

	const [isPending, setIsPending] = useDownloadPending(isDownloaded);

	const hasStartedDownloading = useRef(false);
	useEffect(() => {
		if (isDownloading) hasStartedDownloading.current = true;
		if (!isPending) hasStartedDownloading.current = false;
	}, [isDownloading, isPending]);
	const isUnpacking = isPending && !isDownloading && hasStartedDownloading.current;

	const handleDownload = useCallback(async () => {
		setIsPending(true);
		try {
			logger.info(`ModelRow: downloading ${dir}/${files.map((f) => f.filename).join(', ')}`);
			for (const file of files) {
				await modelService.downloadFile(dir, file.filename, file.url);
			}
			logger.info(`ModelRow: download complete for ${dir}/${id}`);
			onRefresh();
		} catch (err) {
			logger.info(`ModelRow: download failed for ${dir}/${id}: ${String(err)}`);
			setIsPending(false);
		}
	}, [files, dir, onRefresh, id]);

	const handleDelete = useCallback(async () => {
		logger.info(`ModelRow: deleting ${dir}/${id}`);
		for (const file of files) {
			if (downloadedFiles.includes(file.filename)) {
				await modelService.deleteFile(dir, file.filename);
			}
		}
		logger.info(`ModelRow: delete complete for ${dir}/${id}`);
		onRefresh();
	}, [files, dir, downloadedFiles, onRefresh, id]);

	return (
		<div className="flex items-center gap-3 px-4 py-2.5">
			<span
				className={clsx(
					'w-2 h-2 shrink-0 rounded-full',
					isDownloaded ? 'bg-success' : 'bg-text-faint/30',
				)}
			/>
			<span className="flex-1 text-sm text-text truncate flex items-center gap-1.5">
				{label}
				{recommended && (
					<Badge variant="primary" icon={<Star className="w-3 h-3" />}>
						Recommended
					</Badge>
				)}
			</span>
			<span className="text-xs text-text-faint tabular-nums shrink-0">
				{sizeMb >= 1000 ? `${(sizeMb / 1000).toFixed(1)} GB` : `${String(sizeMb)} MB`}
			</span>

			{isDownloading ?
				<DownloadProgressBar progress={progress} />
			: isUnpacking ?
				<Badge variant="neutral" icon={<Package className="w-3 h-3" />}>
					Unpacking
				</Badge>
			: isPending ?
				<Badge variant="neutral" icon={<Loader2 className="w-3 h-3 animate-spin" />}>
					Queued
				</Badge>
			: isDownloaded ?
				<div className="flex items-center gap-1.5">
					{isActive ?
						<Badge variant="primary" icon={<Check className="w-3 h-3" />}>
							Using
						</Badge>
					:	<Button
							type="button"
							variant="ghost"
							size="sm"
							onClick={() => {
								onUse(id);
							}}
						>
							Use
						</Button>
					}
					<Button
						type="button"
						variant="ghost-destructive"
						size="sm"
						onClick={() => void handleDelete()}
					>
						Delete
					</Button>
				</div>
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
