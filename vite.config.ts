import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import { resolve } from 'path';
import { readFileSync } from 'node:fs';

const host = process.env.TAURI_DEV_HOST ?? false;
const { version } = JSON.parse(readFileSync('./package.json', 'utf-8')) as {
	version: string;
};

export default defineConfig(() => ({
	plugins: [tailwindcss(), react()],
	resolve: {
		tsconfigPaths: true,
	},
	define: {
		__APP_VERSION__: JSON.stringify(version),
	},
	clearScreen: false,
	build: {
		rollupOptions: {
			input: {
				main: resolve(__dirname, 'index.html'),
				overlay: resolve(__dirname, 'overlay.html'),
			},
		},
	},
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
