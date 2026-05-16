import { invoke } from '@tauri-apps/api/core';

export interface SystemInfo {
	total_ram_mb: number;
	cpu_count: number;
}

export const systemService = {
	getSystemInfo: () => invoke<SystemInfo>('get_system_info'),
};
