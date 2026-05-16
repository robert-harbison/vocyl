import type { SttModelSize } from '@models/settings';

/**
 * Recommends a Whisper STT model tier based on total system RAM.
 *
 * Thresholds:
 *  < 4 GB  → tiny   (~75 MB)
 *  < 8 GB  → base   (~142 MB)
 *  < 16 GB → small  (~466 MB)
 *  ≥ 16 GB → medium (~1.5 GB)
 */
export function getRecommendedSttModel(totalRamGb: number): SttModelSize {
	if (totalRamGb < 4) return 'tiny';
	if (totalRamGb < 8) return 'base';
	if (totalRamGb < 16) return 'small';
	return 'medium';
}
