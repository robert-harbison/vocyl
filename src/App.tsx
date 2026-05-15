import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useSettingsContext } from '@contexts/SettingsContext';
import SetupScreen from '@screens/SetupScreen';
import SettingsScreen from '@screens/SettingsScreen';
import Loader from '@components/Loader';
import Toast from '@components/Toast';
import { logger } from '@services/logService';

function App() {
	const { loading, exists } = useSettingsContext();
	const [showDuplicateWarning, setShowDuplicateWarning] = useState(false);

	useEffect(() => {
		logger.info('App: UI initialised');

		const unlisten = listen('single-instance-attempted', () => {
			logger.warn('App: duplicate instance launch attempted');
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
