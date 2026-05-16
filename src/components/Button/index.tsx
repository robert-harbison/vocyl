import type { ButtonHTMLAttributes, JSX } from 'react';
import { clsx } from 'clsx';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
	loading?: boolean;
	fullWidth?: boolean;
	variant?: 'primary' | 'secondary' | 'ghost' | 'destructive' | 'ghost-destructive';
	size?: 'sm' | 'md';
}

export default function Button(props: ButtonProps): JSX.Element {
	const {
		loading = false,
		fullWidth = false,
		variant = 'primary',
		size = 'md',
		disabled,
		children,
		className = '',
		...rest
	}: ButtonProps = props;

	const base =
		'inline-flex items-center justify-center gap-1.5 font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-bg-base disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer';

	const sizes = {
		sm: 'text-xs py-1 px-2.5 rounded',
		md: 'text-sm py-2 px-5 rounded-lg',
	};

	const variants = {
		primary: 'bg-primary hover:bg-primary-hover text-white focus:ring-primary',
		secondary:
			'bg-transparent border border-primary/40 hover:border-primary/70 text-primary hover:bg-primary/10 focus:ring-primary',
		ghost: 'bg-transparent hover:bg-bg-elevated text-text-faint hover:text-text focus:ring-primary',
		destructive:
			'bg-transparent hover:bg-error/10 text-error border border-error/40 hover:border-error/60 focus:ring-error',
		'ghost-destructive':
			'bg-transparent hover:bg-error/10 text-error/70 hover:text-error focus:ring-error',
	};

	return (
		<button
			disabled={disabled ?? loading}
			className={clsx(base, sizes[size], variants[variant], fullWidth && 'w-full', className)}
			{...rest}
		>
			{loading ? 'Saving...' : children}
		</button>
	);
}
