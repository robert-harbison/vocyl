import type { SttModelSize } from '@models/settings';

export interface ModelFile {
	filename: string;
	url: string;
}

export interface SttModel {
	id: SttModelSize;
	label: string;
	sizeMb: number;
	file: ModelFile;
}

export const KOKORO_VOICES = [
	{ value: 'af', label: 'Default (US, Female)' },
	{ value: 'af_bella', label: 'Bella (US, Female)' },
	{ value: 'af_nicole', label: 'Nicole (US, Female)' },
	{ value: 'af_sarah', label: 'Sarah (US, Female)' },
	{ value: 'af_sky', label: 'Sky (US, Female)' },
	{ value: 'am_adam', label: 'Adam (US, Male)' },
	{ value: 'am_michael', label: 'Michael (US, Male)' },
	{ value: 'bf_emma', label: 'Emma (GB, Female)' },
	{ value: 'bf_isabella', label: 'Isabella (GB, Female)' },
	{ value: 'bm_george', label: 'George (GB, Male)' },
	{ value: 'bm_lewis', label: 'Lewis (GB, Male)' },
];

export const STT_MODELS: SttModel[] = [
	{
		id: 'tiny',
		label: 'Tiny (~75 MB)',
		sizeMb: 75,
		file: {
			filename: 'ggml-tiny.bin',
			url: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin',
		},
	},
	{
		id: 'base',
		label: 'Base (~142 MB)',
		sizeMb: 142,
		file: {
			filename: 'ggml-base.bin',
			url: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin',
		},
	},
	{
		id: 'small',
		label: 'Small (~466 MB)',
		sizeMb: 466,
		file: {
			filename: 'ggml-small.bin',
			url: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin',
		},
	},
	{
		id: 'medium',
		label: 'Medium (~1.5 GB)',
		sizeMb: 1500,
		file: {
			filename: 'ggml-medium.bin',
			url: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin',
		},
	},
	{
		id: 'large-v3',
		label: 'Large v3 (~3.1 GB)',
		sizeMb: 3100,
		file: {
			filename: 'ggml-large-v3.bin',
			url: 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin',
		},
	},
];
