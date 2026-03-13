import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { SessionInfo } from '$lib/types';

export const sessions = writable<Map<string, SessionInfo>>(new Map());

export const questionCount = derived(sessions, ($sessions) => {
  let count = 0;
  for (const session of $sessions.values()) {
    if (session.state === 'Question') count++;
  }
  return count;
});

export function getSessionForWorktree(
  sessionsMap: Map<string, SessionInfo>,
  worktreePath: string,
): SessionInfo | null {
  for (const session of sessionsMap.values()) {
    if (session.worktree_path === worktreePath) {
      return session;
    }
  }
  return null;
}

let unlistenFn: (() => void) | null = null;

export async function initSessionListener(): Promise<void> {
  if (unlistenFn) return;

  // Load initial sessions
  try {
    const allSessions = await invoke<Record<string, SessionInfo>>('get_all_sessions');
    sessions.set(new Map(Object.entries(allSessions)));
  } catch {
    // Session manager might not have data yet
  }

  // Listen for real-time updates
  unlistenFn = await listen<SessionInfo>('session-update', (event) => {
    sessions.update((map) => {
      const session = event.payload;
      // Use worktree_path as key since that maps to worktrees
      map.set(session.worktree_path, session);
      return new Map(map);
    });
  });
}

export function cleanupSessionListener(): void {
  if (unlistenFn) {
    unlistenFn();
    unlistenFn = null;
  }
}
