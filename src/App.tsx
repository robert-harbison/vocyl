import { useSettingsContext } from '@contexts/SettingsContext';
import SetupScreen from '@screens/SetupScreen';
import SettingsScreen from '@screens/SettingsScreen';
import Loader from '@components/Loader';

function App() {
	const { loading, exists } = useSettingsContext();

	if (loading) {
		return (
			<div className="min-h-screen bg-bg-base flex items-center justify-center">
				<Loader />
			</div>
		);
	}

	if (!exists) {
		return <SetupScreen />;
	}

	return <SettingsScreen />;
}

export default App;
