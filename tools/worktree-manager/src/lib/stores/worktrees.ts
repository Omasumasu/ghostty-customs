import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { WorktreeInfo } from '$lib/types';

export const worktrees = writable<WorktreeInfo[]>([]);
export const selectedWorktree = writable<WorktreeInfo | null>(null);
export const repoPath = writable<string>('');
export const loading = writable<boolean>(false);
export const error = writable<string | null>(null);

export async function loadWorktrees(path: string): Promise<void> {
  loading.set(true);
  error.set(null);
  try {
    const result = await invoke<WorktreeInfo[]>('list_worktrees', { repoPath: path });
    worktrees.set(result);
    repoPath.set(path);
  } catch (e) {
    error.set(String(e));
  } finally {
    loading.set(false);
  }
}

export async function createWorktree(
  repo: string,
  branch: string,
  worktreePath: string,
): Promise<WorktreeInfo> {
  const result = await invoke<WorktreeInfo>('create_worktree', {
    repoPath: repo,
    branch,
    path: worktreePath,
  });
  await loadWorktrees(repo);
  return result;
}

export async function removeWorktree(repo: string, path: string): Promise<void> {
  await invoke<void>('remove_worktree', { repoPath: repo, path });
  await loadWorktrees(repo);
}
