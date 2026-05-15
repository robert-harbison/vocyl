import { StrictMode, useState, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import { listen } from '@tauri-apps/api/event';
import './Global.css';
import './Overlay.css';
import OverlayIndicator, { type OverlayState } from '@components/OverlayIndicator';
import ErrorBoundary from '@components/ErrorBoundary';
import { installGlobalErrorHandlers } from './utils/globalErrorHandlers';

// Register window.onerror + unhandledrejection before React renders anything.
installGlobalErrorHandlers();

function OverlayApp() {
	const [state, setState] = useState<OverlayState>('listening');

	useEffect(() => {
		const unlistenPromise = listen<OverlayState>('overlay-state', (event) => {
			setState(event.payload);
		});
		return () => {
			void unlistenPromise.then((fn) => fn());
		};
	}, []);

	return <OverlayIndicator state={state} />;
}

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<ErrorBoundary context="OverlayWindow">
			<OverlayApp />
		</ErrorBoundary>
	</StrictMode>,
);
