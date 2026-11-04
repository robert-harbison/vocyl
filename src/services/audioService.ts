import { invoke } from '@tauri-apps/api/core';

export interface AudioDevice {
	id: string;
	name: string;
}

export const audioService = {
	listInputDevices: () => invoke<AudioDevice[]>('list_input_devices'),
	listOutputDevices: () => invoke<AudioDevice[]>('list_output_devices'),
};
