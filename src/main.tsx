import React from 'react';
import ReactDOM from 'react-dom/client';
import './Global.css';
import App from './App';
import { SettingsProvider } from '@contexts/SettingsContext';
import ErrorBoundary from '@components/ErrorBoundary';
import { installGlobalErrorHandlers } from './utils/globalErrorHandlers';

// Register window.onerror + unhandledrejection before React renders anything.
installGlobalErrorHandlers();

ReactDOM.createRoot(document.getElementById('root')!).render(
	<React.StrictMode>
		<ErrorBoundary context="SettingsWindow">
			<SettingsProvider>
				<App />
			</SettingsProvider>
		</ErrorBoundary>
	</React.StrictMode>,
);
