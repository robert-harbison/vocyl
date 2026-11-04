import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './Global.css';
import OverlayApp from '@/OverlayApp.tsx';
import ErrorBoundary from '@components/ErrorBoundary';
import { installGlobalErrorHandlers } from './utils/globalErrorHandlers';

// Register window.onerror + unhandledrejection before React renders anything.
installGlobalErrorHandlers();

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<ErrorBoundary context="OverlayWindow">
			<OverlayApp />
		</ErrorBoundary>
	</StrictMode>,
);
