import { useState } from 'react';
import { SETTINGS_SCHEMA_VERSION } from '../types/settings';
import type { VocylSettings } from '../types/settings';

interface Props {
	onComplete: (settings: VocylSettings) => Promise<void>;
}

export default function SetupScreen({ onComplete }: Props) {
	const [testProperty, setTestProperty] = useState('');
	const [saving, setSaving] = useState(false);
	const [saveError, setSaveError] = useState<string | null>(null);

	const handleSubmit = async (e: React.SubmitEvent<HTMLFormElement>) => {
		e.preventDefault();
		setSaving(true);
		setSaveError(null);
		try {
			await onComplete({
				schemaVersion: SETTINGS_SCHEMA_VERSION,
				testProperty,
			});
		} catch (err) {
			setSaveError(String(err));
		} finally {
			setSaving(false);
		}
	};

	return (
		<div className="min-h-screen bg-gray-950 flex items-center justify-center p-6">
			<div className="w-full max-w-md">
				{/* Logo / brand */}
				<div className="text-center mb-10">
					<div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-indigo-600 mb-4">
						<svg
							className="w-8 h-8 text-white"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							strokeWidth={2}
						>
							<path
								strokeLinecap="round"
								strokeLinejoin="round"
								d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
							/>
						</svg>
					</div>
					<h1 className="text-3xl font-bold text-white tracking-tight">
						Welcome to Vocyl
					</h1>
					<p className="mt-2 text-gray-400 text-sm">
						Let's get you set up. You can change these settings any
						time from the tray icon.
					</p>
				</div>

				{/* Setup form */}
				<form
					onSubmit={(e) => void handleSubmit(e)}
					className="bg-gray-900 rounded-2xl border border-gray-800 p-6 space-y-6"
				>
					<div>
						<label
							htmlFor="testProperty"
							className="block text-sm font-medium text-gray-300 mb-1.5"
						>
							Test Property
						</label>
						<input
							id="testProperty"
							type="text"
							value={testProperty}
							onChange={(e) => {
								setTestProperty(e.target.value);
							}}
							placeholder="Enter a value…"
							className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
						/>
						<p className="mt-1 text-xs text-gray-500">
							This is a placeholder field — more settings will
							appear here.
						</p>
					</div>

					{saveError && (
						<p className="text-xs text-red-400 bg-red-900/20 border border-red-800 rounded-lg px-3 py-2">
							{saveError}
						</p>
					)}

					<button
						type="submit"
						disabled={saving}
						className="w-full py-2.5 px-4 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-gray-900"
					>
						{saving ? 'Saving…' : 'Get Started'}
					</button>
				</form>
			</div>
		</div>
	);
}
