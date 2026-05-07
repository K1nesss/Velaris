import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater';

export type AppUpdate = Update;
export type AppUpdateDownloadEvent = DownloadEvent;

export async function checkForAppUpdate() {
	return await check({
		timeout: 10000
	});
}

export function formatUpdateNotes(body?: string) {
	return body?.trim() || '';
}
