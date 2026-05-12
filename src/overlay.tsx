import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import OverlayIndicator from './OverlayIndicator';

createRoot(document.getElementById('root')!).render(
	<StrictMode>
		<OverlayIndicator />
	</StrictMode>,
);
