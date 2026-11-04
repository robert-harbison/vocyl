import { invoke } from '@tauri-apps/api/core';

export const appService = {
	exit: () => invoke('exit_app'),
};
