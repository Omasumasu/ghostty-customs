<script lang="ts">
  import type { WorktreeInfo, SessionInfo } from '$lib/types';
  import { getSessionForWorktree } from '$lib/stores/sessions';

  let {
    worktrees,
    sessions,
    selected,
    onselect,
    onnewclick,
    oncleanupclick,
  }: {
    worktrees: WorktreeInfo[];
    sessions: Map<string, SessionInfo>;
    selected: WorktreeInfo | null;
    onselect: (wt: WorktreeInfo) => void;
    onnewclick: () => void;
    oncleanupclick: () => void;
  } = $props();

  function stateColor(state: string | undefined): string {
    switch (state) {
      case 'Working': return 'border-tn-green';
      case 'Question': return 'border-tn-red';
      case 'Merged': return 'border-tn-purple';
      default: return 'border-tn-fg-muted';
    }
  }

  function stateDot(state: string | undefined): string {
    switch (state) {
      case 'Working': return 'bg-tn-green';
      case 'Question': return 'bg-tn-red';
      case 'Merged': return 'bg-tn-purple';
      default: return 'bg-tn-fg-muted';
    }
  }

  function branchShort(branch: string): string {
    const parts = branch.split('/');
    return parts[parts.length - 1];
  }
</script>

<aside class="w-[280px] min-w-[280px] h-full bg-tn-bg-dark border-r border-tn-border flex flex-col">
  <div class="flex items-center justify-between px-3 py-2 border-b border-tn-border">
    <span class="text-tn-fg-dim text-xs uppercase tracking-wider">Worktrees</span>
    <div class="flex gap-1">
      <button
        onclick={oncleanupclick}
        class="px-2 py-0.5 text-xs text-tn-fg-muted hover:text-tn-yellow hover:bg-tn-bg-highlight rounded transition-colors"
        title="Cleanup merged"
      >
        Cleanup
      </button>
      <button
        onclick={onnewclick}
        class="px-2 py-0.5 text-xs text-tn-blue hover:bg-tn-bg-highlight rounded transition-colors"
      >
        + New
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each worktrees as wt (wt.path)}
      {@const session = getSessionForWorktree(sessions, wt.path)}
      {@const isSelected = selected?.path === wt.path}
      <button
        onclick={() => onselect(wt)}
        class="w-full text-left px-3 py-2 border-l-2 transition-colors {stateColor(session?.state)} {isSelected ? 'bg-tn-bg-highlight' : 'hover:bg-tn-bg-alt'}"
      >
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full shrink-0 {stateDot(session?.state)}"></span>
          <span class="text-sm truncate {isSelected ? 'text-tn-fg' : 'text-tn-fg-dim'}">
            {branchShort(wt.branch)}
          </span>
        </div>
        {#if wt.is_bare}
          <span class="text-[10px] text-tn-fg-muted ml-4">(bare)</span>
        {/if}
      </button>
    {/each}

    {#if worktrees.length === 0}
      <div class="px-3 py-4 text-center text-tn-fg-muted text-xs">
        No worktrees found
      </div>
    {/if}
  </div>
</aside>
