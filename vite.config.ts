import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST ?? false;

export default defineConfig(() => ({
	plugins: [react()],
	clearScreen: false,
	test: {
		environment: 'jsdom',
		globals: true,
		setupFiles: ['src/test/setup.ts'],
		exclude: ['**/node_modules/**', '**/src-tauri/**'],
	},
	server: {
		port: 1420,
		strictPort: true,
		host: host,
		hmr:
			host ?
				{
					protocol: 'ws',
					host,
					port: 1421,
				}
			:	undefined,
		watch: {
			ignored: ['**/src-tauri/**'],
		},
	},
}));
