import React from 'react';
import styles from './OverlayIndicator.module.css';

export type OverlayState = 'waiting' | 'listening' | 'processing' | 'reading' | 'no-text' | 'error';

interface Props {
	state?: OverlayState;
}

const STATE_LABEL: Record<OverlayState, string> = {
	waiting: 'Waiting…',
	listening: 'Listening…',
	processing: 'Processing…',
	reading: 'Reading…',
	'no-text': 'No text selected',
	error: 'Error',
};

const STATE_COLOR: Record<OverlayState, string> = {
	waiting: '#fbbf24', // amber-400
	listening: '#818cf8', // indigo-400
	processing: '#60a5fa', // blue-400
	reading: '#4ade80', // green-400
	'no-text': '#facc15', // yellow-400
	error: '#f87171', // red-400
};

// Max height (%) for each bar — varies so bars are naturally different sizes.
const BAR_HEIGHTS = [
	52, 72, 88, 62, 82, 68, 94, 58, 78, 90, 72, 62, 84, 94, 68, 78, 56, 88, 72, 82, 60, 94, 68, 78,
	52, 72,
];

export default function OverlayIndicator({ state = 'listening' }: Props) {
	const color = STATE_COLOR[state];

	return (
		<div className={styles.container} style={{ '--wave-color': color } as React.CSSProperties}>
			{/* pill clips everything to its rounded shape */}
			<div className={styles.pill}>
				<div className={styles.waves}>
					{BAR_HEIGHTS.map((h, i) => (
						<div
							key={i}
							className={styles.bar}
							style={{
								height: `${h}%`,
								// Negative delay starts each bar mid-animation so they
								// never all move in sync on first render.
								animationDelay: `${-(((i * 113) % 1300) / 1000)}s`,
								animationDuration: `${0.75 + (i % 7) * 0.09}s`,
							}}
						/>
					))}
				</div>

				{/* text sits directly over the waves — no blocking background */}
				<div className={styles.labelWrap}>
					<span className={styles.label}>{STATE_LABEL[state]}</span>
				</div>
			</div>
		</div>
	);
}
