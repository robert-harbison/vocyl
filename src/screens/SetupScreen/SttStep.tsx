import type { JSX } from 'react';
import Button from '@components/Button';
import ModelRow, { type DownloadStates } from '@screens/SettingsScreen/ModelRow';
import { STT_MODELS } from '@/constants/models';
import type { SttModelSize } from '@models/settings';

interface SttStepProps {
	sttFiles: string[];
	dlStates: DownloadStates;
	selectedModel: SttModelSize | null;
	recommendedModelId: SttModelSize | null;
	systemRamGb: number | null;
	onSelectModel: (id: SttModelSize) => void;
	onRefresh: () => void;
	onNext: () => void;
	onSkip: () => void;
}

export default function SttStep({
	sttFiles,
	dlStates,
	selectedModel,
	recommendedModelId,
	systemRamGb,
	onSelectModel,
	onRefresh,
	onNext,
	onSkip,
}: SttStepProps): JSX.Element {
	return (
		<div className="space-y-6">
			<div className="text-center">
				<h2 className="text-2xl font-bold text-text tracking-tight">
					Speech-to-Text Model
				</h2>
				<p className="mt-1 text-sm text-text-subtle max-w-sm mx-auto">
					Choose a Whisper model for push-to-talk transcription.
					{systemRamGb !== null && (
						<>
							{' '}
							Based on your{' '}
							<span className="font-medium text-text-muted">
								{systemRamGb.toFixed(0)} GB RAM
							</span>
							, we recommend the highlighted model.
						</>
					)}
				</p>
			</div>

			<div className="bg-bg-surface rounded-xl border border-border divide-y divide-border">
				{STT_MODELS.map((model) => (
					<ModelRow
						key={model.id}
						id={model.id}
						label={model.label}
						sizeMb={model.sizeMb}
						files={[model.file]}
						dir="stt"
						downloadedFiles={sttFiles}
						dlStates={dlStates}
						activeId={selectedModel}
						recommended={recommendedModelId === model.id}
						onUse={(id) => {
							onSelectModel(id as SttModelSize);
						}}
						onRefresh={onRefresh}
					/>
				))}
			</div>

			<div className="flex items-center justify-between">
				<button
					type="button"
					onClick={onSkip}
					className="text-sm text-text-faint hover:text-text transition-colors"
				>
					Skip for now
				</button>
				<Button onClick={onNext}>Continue →</Button>
			</div>
		</div>
	);
}
