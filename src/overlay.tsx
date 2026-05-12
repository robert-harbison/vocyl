import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './Global.css';
import OverlayIndicator from '@components/OverlayIndicator';

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<OverlayIndicator />
	</StrictMode>,
);
