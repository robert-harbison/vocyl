import type { ButtonHTMLAttributes, JSX } from 'react';
import { clsx } from 'clsx';

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
	loading?: boolean;
	fullWidth?: boolean;
	variant?: 'primary' | 'destructive';
}

export default function Button(props: ButtonProps): JSX.Element {
	const {
		loading = false,
		fullWidth = false,
		variant = 'primary',
		disabled,
		children,
		className = '',
		...rest
	}: ButtonProps = props;

	const base =
		'rounded-lg text-sm font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';

	const variants = {
		primary:
			'bg-primary hover:bg-primary-hover text-text focus:ring-primary focus:ring-offset-bg-base py-2 px-5',
		destructive:
			'bg-transparent hover:bg-error/10 text-error border border-error/40 focus:ring-error focus:ring-offset-bg-base py-2 px-5',
	};

	return (
		<button
			disabled={disabled ?? loading}
			className={clsx(base, variants[variant], fullWidth && 'w-full', className)}
			{...rest}
		>
			{loading ? 'Saving...' : children}
		</button>
	);
}
