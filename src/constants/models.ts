import type { SttModelSize } from '@models/settings';

export interface ModelFile {
	filename: string;
	url: string;
}

export interface KokoroModel {
	id: string;
	label: string;
	sizeMb: number;
	files: ModelFile[];
}

export interface PiperVoice {
	id: string;
	label: string;
	sizeMb: number;
	files: ModelFile[];
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

// Kokoro is downloaded as a single self-contained package via download_kokoro_package.
// The KOKORO_MODELS array is kept for display purposes only (size label etc.).
export const KOKORO_MODELS: KokoroModel[] = [
	{
		id: 'kokoro-en-v0_19',
		label: 'Kokoro English v0.19',
		sizeMb: 90,
		files: [], // package download handled by download_kokoro_package command
	},
];

export const PIPER_VOICES: PiperVoice[] = [
	{
		id: 'en_US-lessac-medium',
		label: 'Lessac (US, Female)',
		sizeMb: 63,
		files: [
			{
				filename: 'en_US-lessac-medium.onnx',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx',
			},
			{
				filename: 'en_US-lessac-medium.onnx.json',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json',
			},
		],
	},
	{
		id: 'en_US-ryan-medium',
		label: 'Ryan (US, Male)',
		sizeMb: 63,
		files: [
			{
				filename: 'en_US-ryan-medium.onnx',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/ryan/medium/en_US-ryan-medium.onnx',
			},
			{
				filename: 'en_US-ryan-medium.onnx.json',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/ryan/medium/en_US-ryan-medium.onnx.json',
			},
		],
	},
	{
		id: 'en_US-amy-medium',
		label: 'Amy (US, Female)',
		sizeMb: 63,
		files: [
			{
				filename: 'en_US-amy-medium.onnx',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/amy/medium/en_US-amy-medium.onnx',
			},
			{
				filename: 'en_US-amy-medium.onnx.json',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/amy/medium/en_US-amy-medium.onnx.json',
			},
		],
	},
	{
		id: 'en_GB-alan-medium',
		label: 'Alan (GB, Male)',
		sizeMb: 63,
		files: [
			{
				filename: 'en_GB-alan-medium.onnx',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/alan/medium/en_GB-alan-medium.onnx',
			},
			{
				filename: 'en_GB-alan-medium.onnx.json',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/alan/medium/en_GB-alan-medium.onnx.json',
			},
		],
	},
	{
		id: 'en_GB-jenny_dioco-medium',
		label: 'Jenny (GB, Female)',
		sizeMb: 63,
		files: [
			{
				filename: 'en_GB-jenny_dioco-medium.onnx',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/jenny_dioco/medium/en_GB-jenny_dioco-medium.onnx',
			},
			{
				filename: 'en_GB-jenny_dioco-medium.onnx.json',
				url: 'https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_GB/jenny_dioco/medium/en_GB-jenny_dioco-medium.onnx.json',
			},
		],
	},
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
