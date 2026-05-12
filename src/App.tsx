import './Global.css';
import { useSettings } from './hooks/useSettings';
import SetupScreen from './components/SetupScreen';
import SettingsScreen from './components/SettingsScreen';
import type { VocylSettings } from './types/settings';

function App() {
	const { settings, saveSettings, loading, exists, setExists } =
		useSettings();

	if (loading) {
		return (
			<div className="min-h-screen bg-gray-950 flex items-center justify-center">
				<div className="w-6 h-6 rounded-full border-2 border-indigo-500 border-t-transparent animate-spin" />
			</div>
		);
	}

	if (!exists) {
		const handleSetupComplete = async (initial: VocylSettings) => {
			await saveSettings(initial);
			setExists(true);
		};
		return <SetupScreen onComplete={handleSetupComplete} />;
	}

	return <SettingsScreen settings={settings} onSave={saveSettings} />;
}

export default App;
