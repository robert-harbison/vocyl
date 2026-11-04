import { useEffect, useState } from 'react';
import { TriangleAlert } from 'lucide-react';

interface ToastProps {
	message: string;
	duration?: number;
	onDismiss: () => void;
}

export default function Toast({ message, duration = 4000, onDismiss }: ToastProps) {
	const [visible, setVisible] = useState(true);

	useEffect(() => {
		const timer = setTimeout(() => {
			setVisible(false);
		}, duration);
		return () => {
			clearTimeout(timer);
		};
	}, [duration]);

	useEffect(() => {
		if (!visible) {
			const fadeTimer = setTimeout(onDismiss, 300);
			return () => {
				clearTimeout(fadeTimer);
			};
		}
	}, [visible, onDismiss]);

	return (
		<div
			className={`fixed bottom-5 left-1/2 -translate-x-1/2 flex items-center gap-3 px-4 py-3 rounded-lg bg-bg-surface border border-border-input text-text-muted text-sm shadow-lg transition-opacity duration-300 ${visible ? 'opacity-100' : 'opacity-0'}`}
		>
			<TriangleAlert className="w-4 h-4 text-yellow-400 shrink-0" />
			{message}
		</div>
	);
}
