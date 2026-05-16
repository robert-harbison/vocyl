import type { JSX, ReactNode } from 'react';
import { clsx } from 'clsx';

interface BadgeProps {
	children: ReactNode;
	variant?: 'primary' | 'success' | 'error' | 'neutral';
	icon?: ReactNode;
}

export default function Badge({ children, variant = 'neutral', icon }: BadgeProps): JSX.Element {
	const variants = {
		primary: 'bg-primary/10 text-primary',
		success: 'bg-success/10 text-success',
		error: 'bg-error/10 text-error',
		neutral: 'bg-bg-elevated text-text-faint',
	};

	return (
		<span
			className={clsx(
				'inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded',
				variants[variant],
			)}
		>
			{icon}
			{children}
		</span>
	);
}
