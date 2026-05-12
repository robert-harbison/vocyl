import styles from './OverlayIndicator.module.css';

export default function OverlayIndicator() {
	return (
		<div className={styles.pill}>
			<span className={styles.dot} />
			<span className={styles.label}>Listening...</span>
		</div>
	);
}
