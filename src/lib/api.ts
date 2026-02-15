import { invoke } from '@tauri-apps/api/core';

export async function hello() {
  return await invoke<string>('hello');
}
