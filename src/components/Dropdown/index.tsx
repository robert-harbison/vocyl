import { useState, useRef, useEffect, type JSX } from 'react';
import { clsx } from 'clsx';

export interface DropdownOption<T extends string = string> {
	value: T;
	label: string;
}

interface DropdownProps<T extends string = string> {
	options: DropdownOption<T>[];
	value: T | null;
	onChange: (value: T) => void;
	placeholder?: string;
	disabled?: boolean;
	className?: string;
}

export default function Dropdown<T extends string = string>({
	options,
	value,
	onChange,
	placeholder = 'Select…',
	disabled = false,
	className,
}: DropdownProps<T>): JSX.Element {
	const [open, setOpen] = useState(false);
	const ref = useRef<HTMLDivElement>(null);

	const selected = options.find((o) => o.value === value);

	useEffect(() => {
		const onMouseDown = (e: MouseEvent) => {
			if (ref.current && !ref.current.contains(e.target as Node)) {
				setOpen(false);
			}
		};
		document.addEventListener('mousedown', onMouseDown);
		return () => document.removeEventListener('mousedown', onMouseDown);
	}, []);

	return (
		<div ref={ref} className={clsx('relative', className)}>
			<button
				type="button"
				disabled={disabled}
				className={clsx(
					'w-full flex items-center justify-between gap-2',
					'rounded-lg bg-bg-elevated border border-border-input px-3 py-1.5',
					'text-sm transition focus:outline-none focus:ring-2 focus:ring-primary',
					disabled ?
						'cursor-not-allowed opacity-50 text-text-faint'
					:	'cursor-pointer text-text hover:border-border',
				)}
				onClick={() => setOpen((o) => !o)}
			>
				<span className={clsx(!selected && 'text-text-faint')}>
					{selected ? selected.label : placeholder}
				</span>
				<svg
					className={clsx(
						'w-4 h-4 shrink-0 text-text-faint transition-transform duration-150',
						open && 'rotate-180',
					)}
					viewBox="0 0 20 20"
					fill="currentColor"
				>
					<path
						fillRule="evenodd"
						d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
						clipRule="evenodd"
					/>
				</svg>
			</button>

			{open && !disabled && (
				<div className="absolute top-full left-0 right-0 mt-1 z-50 rounded-lg border border-border bg-bg-surface shadow-lg overflow-hidden">
					<div className="max-h-48 overflow-y-auto py-1">
						{options.length === 0 ?
							<div className="px-3 py-2 text-sm text-text-faint text-center">
								No options available
							</div>
						:	options.map((opt) => (
								<button
									key={opt.value}
									type="button"
									className={clsx(
										'w-full text-left px-3 py-2 text-sm transition-colors',
										opt.value === value ?
											'bg-primary/10 text-primary'
										:	'text-text hover:bg-bg-elevated',
									)}
									onClick={() => {
										onChange(opt.value);
										setOpen(false);
									}}
								>
									{opt.label}
								</button>
							))
						}
					</div>
				</div>
			)}
		</div>
	);
}
