import { Component, type ErrorInfo, type ReactNode } from 'react';
import { logger } from '@services/logService';

interface Props {
	children: ReactNode;
	/** Optional label shown in the UI and included in the log message. */
	context?: string;
}

interface State {
	hasError: boolean;
	message: string;
}

/**
 * ErrorBoundary — catches unhandled React render/lifecycle errors and logs
 * them via the structured logger so they appear in the app log file.
 *
 * Wraps the root of both the main window and the overlay window.
 */
export default class ErrorBoundary extends Component<Props, State> {
	constructor(props: Props) {
		super(props);
		this.state = { hasError: false, message: '' };
	}

	static getDerivedStateFromError(error: unknown): State {
		const message = error instanceof Error ? error.message : String(error);
		return { hasError: true, message };
	}

	override componentDidCatch(error: unknown, info: ErrorInfo): void {
		const ctx = this.props.context ?? 'App';
		const message = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
		const stack = info.componentStack ?? '(no stack)';
		logger.error(
			`[ErrorBoundary:${ctx}] Unhandled render error — ${message}\nComponent stack:${stack}`,
		);
	}

	override render(): ReactNode {
		if (this.state.hasError) {
			return (
				<div
					style={{
						display: 'flex',
						flexDirection: 'column',
						alignItems: 'center',
						justifyContent: 'center',
						height: '100vh',
						padding: '2rem',
						fontFamily: 'sans-serif',
						color: '#e87070',
						background: '#1a1a1a',
						gap: '0.75rem',
					}}
				>
					<p style={{ fontWeight: 600, fontSize: '1rem', margin: 0 }}>
						Something went wrong
					</p>
					<p
						style={{
							fontSize: '0.8rem',
							color: '#888',
							margin: 0,
							maxWidth: '40ch',
							textAlign: 'center',
						}}
					>
						{this.state.message}
					</p>
					<p style={{ fontSize: '0.75rem', color: '#555', margin: 0 }}>
						Check the log file for details.
					</p>
				</div>
			);
		}
		return this.props.children;
	}
}
