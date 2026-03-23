import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const repositories = writable<string[]>([]);
export const selectedRepo = writable<string | null>(null);

export async function loadRepositories(): Promise<void> {
  try {
    const result = await invoke<string[]>('list_repositories');
    repositories.set(result);
  } catch (e) {
    console.error('リポジトリ一覧の取得に失敗しました:', e);
    repositories.set([]);
  }
}

export async function addRepository(path: string): Promise<void> {
  await invoke<void>('add_repository', { path });
  await loadRepositories();
}

export async function removeRepository(path: string): Promise<void> {
  await invoke<void>('remove_repository', { path });
  await loadRepositories();
}
