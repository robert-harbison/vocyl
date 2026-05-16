import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import OverlayIndicator, { type OverlayState } from '@components/OverlayIndicator';

import './Overlay.css';

export default function OverlayApp() {
	const [state, setState] = useState<OverlayState>('listening');

	useEffect(() => {
		const unlistenPromise = listen<OverlayState>('overlay-state', (event) => {
			setState(event.payload);
		});

		return () => {
			void unlistenPromise.then((fn) => {
				fn();
			});
		};
	}, []);

	return <OverlayIndicator state={state} />;
}
