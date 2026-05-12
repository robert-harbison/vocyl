import type { InputHTMLAttributes, Ref } from 'react';
import { clsx } from 'clsx';

interface TextFieldProps extends InputHTMLAttributes<HTMLInputElement> {
	label?: string;
	hint?: string;
	error?: string;
	ref?: Ref<HTMLInputElement>;
}

export default function TextField({
	label,
	hint,
	error,
	id,
	ref,
	className = '',
	...rest
}: TextFieldProps) {
	return (
		<div>
			{label && (
				<label htmlFor={id} className="block text-sm font-medium text-text-muted mb-1.5">
					{label}
				</label>
			)}
			<input
				ref={ref}
				id={id}
				className={clsx(
					'rounded-lg bg-bg-elevated border px-3 py-2 text-sm text-text placeholder-text-faint',
					'focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition',
					error ? 'border-error' : 'border-border-input',
					className,
				)}
				{...rest}
			/>
			{error ?
				<p className="mt-1 text-xs text-error">{error}</p>
			: hint ?
				<p className="mt-1 text-xs text-text-faint">{hint}</p>
			:	null}
		</div>
	);
}
