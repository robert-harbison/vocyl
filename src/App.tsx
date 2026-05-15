import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useSettingsContext } from '@contexts/SettingsContext';
import SetupScreen from '@screens/SetupScreen';
import SettingsScreen from '@screens/SettingsScreen';
import Loader from '@components/Loader';
import Toast from '@components/Toast';

function App() {
	const { loading, exists } = useSettingsContext();
	const [showDuplicateWarning, setShowDuplicateWarning] = useState(false);

	useEffect(() => {
		const unlisten = listen('single-instance-attempted', () => {
			setShowDuplicateWarning(true);
		});
		return () => {
			unlisten.then((fn) => fn());
		};
	}, []);

	if (loading) {
		return (
			<div className="min-h-screen bg-bg-base flex items-center justify-center">
				<Loader />
			</div>
		);
	}

	return (
		<>
			{exists ?
				<SettingsScreen />
			:	<SetupScreen />}
			{showDuplicateWarning && (
				<Toast
					message="Vocyl is already running."
					onDismiss={() => setShowDuplicateWarning(false)}
				/>
			)}
		</>
	);
}

export default App;
