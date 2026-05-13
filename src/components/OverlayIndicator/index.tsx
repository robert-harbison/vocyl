import { clsx } from 'clsx';
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

export default function OverlayIndicator({ state = 'listening' }: Props) {
	return (
		<div className={styles.pill}>
			<span
				className={clsx(
					styles.dot,
					state === 'waiting' && styles.dotAmber,
					state === 'listening' && styles.dotRed,
					state === 'processing' && styles.dotBlue,
					state === 'reading' && styles.dotGreen,
					state === 'no-text' && styles.dotYellow,
					state === 'error' && styles.dotError,
				)}
			/>
			<span className={styles.label}>{STATE_LABEL[state]}</span>
		</div>
	);
}
