import { useForm } from 'react-hook-form';
import { SETTINGS_SCHEMA_VERSION } from '@models/settings';
import { useSettingsContext } from '@contexts/SettingsContext';
import Button from '@components/Button';
import TextField from '@components/TextField';

interface FormValues {
	testProperty: string;
}

export default function SetupScreen() {
	const { saveSettings } = useSettingsContext();

	const {
		register,
		handleSubmit,
		formState: { isSubmitting, errors },
	} = useForm<FormValues>({
		defaultValues: { testProperty: '' },
	});

	const onSubmit = async (data: FormValues) => {
		await saveSettings({
			schemaVersion: SETTINGS_SCHEMA_VERSION,
			testProperty: data.testProperty,
		});
	};

	return (
		<div className="min-h-screen bg-bg-base flex items-center justify-center p-6">
			<div className="w-full max-w-md">
				{/* Logo / brand */}
				<div className="text-center mb-10">
					<div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary mb-4">
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
					<h1 className="text-3xl font-bold text-text tracking-tight">
						Welcome to Vocyl
					</h1>
					<p className="mt-2 text-text-subtle text-sm">
						Let's get you set up. You can change these settings any time from the tray
						icon.
					</p>
				</div>

				{/* Setup form */}
				<form
					onSubmit={(e) => void handleSubmit(onSubmit)(e)}
					className="bg-bg-surface rounded-2xl border border-border p-6 space-y-6"
				>
					<TextField
						{...register('testProperty', { required: 'This field is required' })}
						id="testProperty"
						label="Test Property"
						type="text"
						placeholder="Enter a value..."
						hint="This is a placeholder field — more settings will appear here."
						error={errors.testProperty?.message}
						className="w-full"
					/>

					<Button type="submit" loading={isSubmitting} fullWidth>
						Get Started
					</Button>
				</form>
			</div>
		</div>
	);
}
