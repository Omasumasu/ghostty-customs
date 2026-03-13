<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { WorktreeInfo, SessionInfo, ActivityEntry, GitStatus } from '$lib/types';
  import SessionStatus from './SessionStatus.svelte';
  import ActivityLog from './ActivityLog.svelte';

  let {
    worktree,
    session,
  }: {
    worktree: WorktreeInfo | null;
    session: SessionInfo | null;
  } = $props();

  let gitStatus = $state<GitStatus | null>(null);
  let activities = $state<ActivityEntry[]>([]);
  let loadingStatus = $state(false);

  $effect(() => {
    if (!worktree) {
      gitStatus = null;
      activities = [];
      return;
    }

    const path = worktree.path;
    loadingStatus = true;

    invoke<GitStatus>('get_worktree_status', { worktreePath: path })
      .then((s) => { gitStatus = s; })
      .catch(() => { gitStatus = null; })
      .finally(() => { loadingStatus = false; });

    // Load activity log if session exists
    if (session?.worktree_path === path) {
      invoke<ActivityEntry[]>('get_activity_log', { sessionId: path })
        .then((log) => { activities = log; })
        .catch(() => { activities = []; });
    } else {
      activities = [];
    }
  });

  async function openDifftool() {
    if (!worktree) return;
    try {
      await invoke('open_difftool', { worktreePath: worktree.path });
    } catch (e) {
      console.error('Failed to open difftool:', e);
    }
  }

  async function openIDE() {
    if (!worktree) return;
    try {
      await invoke('open_in_ide', { path: worktree.path });
    } catch (e) {
      console.error('Failed to open IDE:', e);
    }
  }

  async function openTerminal() {
    if (!worktree) return;
    try {
      await invoke('open_terminal', { path: worktree.path });
    } catch (e) {
      console.error('Failed to open terminal:', e);
    }
  }

  function fileCount(status: GitStatus | null): number {
    if (!status) return 0;
    return status.modified.length + status.added.length + status.deleted.length + status.untracked.length;
  }
</script>

{#if worktree}
  <div class="flex-1 h-full overflow-y-auto p-4 space-y-4">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-medium text-tn-fg">{worktree.branch}</h2>
        <p class="text-xs text-tn-fg-muted mt-0.5 font-mono">{worktree.path}</p>
      </div>
      <div class="flex gap-1">
        <button
          onclick={openDifftool}
          class="px-2.5 py-1 text-xs text-tn-fg-dim bg-tn-bg-alt border border-tn-border rounded hover:bg-tn-bg-highlight hover:text-tn-blue transition-colors"
          title="Open difftool"
        >
          Diff
        </button>
        <button
          onclick={openIDE}
          class="px-2.5 py-1 text-xs text-tn-fg-dim bg-tn-bg-alt border border-tn-border rounded hover:bg-tn-bg-highlight hover:text-tn-green transition-colors"
          title="Open in IDE"
        >
          IDE
        </button>
        <button
          onclick={openTerminal}
          class="px-2.5 py-1 text-xs text-tn-fg-dim bg-tn-bg-alt border border-tn-border rounded hover:bg-tn-bg-highlight hover:text-tn-purple transition-colors"
          title="Open Ghostty terminal"
        >
          Terminal
        </button>
      </div>
    </div>

    <!-- Session Status -->
    <SessionStatus {session} />

    <!-- Git Info -->
    <div class="rounded border border-tn-border bg-tn-bg-alt p-3">
      <span class="text-xs text-tn-fg-muted uppercase tracking-wider block mb-2">Git Info</span>
      <div class="grid grid-cols-2 gap-y-1.5 text-xs">
        <span class="text-tn-fg-muted">Branch</span>
        <span class="text-tn-fg-dim">{worktree.branch}</span>
        <span class="text-tn-fg-muted">Commit</span>
        <span class="text-tn-fg-dim font-mono">{worktree.commit.slice(0, 8)}</span>
        <span class="text-tn-fg-muted">Files changed</span>
        <span class="text-tn-fg-dim">
          {#if loadingStatus}
            ...
          {:else}
            {fileCount(gitStatus)}
          {/if}
        </span>
      </div>

      {#if gitStatus && !loadingStatus}
        <div class="mt-2 flex gap-3 text-[10px]">
          {#if gitStatus.modified.length > 0}
            <span class="text-tn-yellow">{gitStatus.modified.length} modified</span>
          {/if}
          {#if gitStatus.added.length > 0}
            <span class="text-tn-green">{gitStatus.added.length} added</span>
          {/if}
          {#if gitStatus.deleted.length > 0}
            <span class="text-tn-red">{gitStatus.deleted.length} deleted</span>
          {/if}
          {#if gitStatus.untracked.length > 0}
            <span class="text-tn-fg-muted">{gitStatus.untracked.length} untracked</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Activity Log -->
    <div class="rounded border border-tn-border bg-tn-bg-alt p-3">
      <ActivityLog entries={activities} />
    </div>
  </div>
{:else}
  <div class="flex-1 h-full flex items-center justify-center">
    <div class="text-center">
      <p class="text-tn-fg-muted text-sm">Select a worktree to view details</p>
    </div>
  </div>
{/if}
