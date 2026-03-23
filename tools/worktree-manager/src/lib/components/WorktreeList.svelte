<script lang="ts">
  import type { WorktreeInfo, SessionInfo } from '$lib/types';
  import { getSessionForWorktree } from '$lib/stores/sessions';
  import { repositories, selectedRepo, addRepository, removeRepository } from '$lib/stores/repositories';
  import { open } from '@tauri-apps/plugin-dialog';

  let {
    worktrees,
    sessions,
    selected,
    onselect,
    onnewclick,
    oncleanupclick,
    onrepochan,
  }: {
    worktrees: WorktreeInfo[];
    sessions: Map<string, SessionInfo>;
    selected: WorktreeInfo | null;
    onselect: (wt: WorktreeInfo) => void;
    onnewclick: () => void;
    oncleanupclick: () => void;
    onrepochan: (path: string) => void;
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

  function repoName(path: string): string {
    const parts = path.replace(/\/$/, '').split('/');
    return parts[parts.length - 1];
  }

  async function handleAddRepo() {
    const selected = await open({
      directory: true,
      title: 'Gitリポジトリを追加',
    });
    if (selected && typeof selected === 'string') {
      await addRepository(selected);
      onrepochan(selected);
    }
  }

  async function handleRemoveRepo(path: string) {
    if (confirm(`リポジトリ「${repoName(path)}」を一覧から削除しますか？\n（ディスク上のファイルは削除されません）`)) {
      await removeRepository(path);
      // If we removed the currently selected repo, switch to first available
      const repos = $repositories;
      if (repos.length > 0) {
        onrepochan(repos[0]);
      } else {
        onrepochan('');
      }
    }
  }
</script>

<aside class="w-[280px] min-w-[280px] h-full bg-tn-bg-dark border-r border-tn-border flex flex-col">

  <!-- リポジトリセレクター -->
  <div class="border-b border-tn-border">
    <div class="flex items-center justify-between px-3 py-2">
      <span class="text-tn-fg-dim text-xs uppercase tracking-wider">リポジトリ</span>
      <button
        onclick={handleAddRepo}
        class="px-2 py-0.5 text-xs text-tn-blue hover:bg-tn-bg-highlight rounded transition-colors"
        title="リポジトリを追加"
      >
        + 追加
      </button>
    </div>

    <div class="pb-1">
      {#each $repositories as repo (repo)}
        {@const isActive = $selectedRepo === repo}
        <div
          class="group flex items-center px-3 py-1.5 cursor-pointer transition-colors {isActive ? 'bg-tn-bg-highlight' : 'hover:bg-tn-bg-alt'}"
        >
          <button
            onclick={() => onrepochan(repo)}
            class="flex-1 text-left truncate"
            title={repo}
          >
            <span class="text-sm {isActive ? 'text-tn-fg' : 'text-tn-fg-dim'}">
              {repoName(repo)}
            </span>
          </button>
          <button
            onclick={() => handleRemoveRepo(repo)}
            class="opacity-0 group-hover:opacity-100 text-tn-fg-muted hover:text-tn-red text-xs px-1 transition-all"
            title="削除"
          >
            ✕
          </button>
        </div>
      {/each}

      {#if $repositories.length === 0}
        <div class="px-3 py-2 text-center text-tn-fg-muted text-xs">
          リポジトリが登録されていません
        </div>
      {/if}
    </div>
  </div>

  <!-- Worktreeリスト -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-tn-border">
    <span class="text-tn-fg-dim text-xs uppercase tracking-wider">Worktrees</span>
    <div class="flex gap-1">
      <button
        onclick={oncleanupclick}
        class="px-2 py-0.5 text-xs text-tn-fg-muted hover:text-tn-yellow hover:bg-tn-bg-highlight rounded transition-colors"
        title="マージ済みを削除"
      >
        整理
      </button>
      <button
        onclick={onnewclick}
        class="px-2 py-0.5 text-xs text-tn-blue hover:bg-tn-bg-highlight rounded transition-colors"
      >
        + 新規
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

    {#if worktrees.length === 0 && $selectedRepo}
      <div class="px-3 py-4 text-center text-tn-fg-muted text-xs">
        Worktree が見つかりません
      </div>
    {/if}

    {#if !$selectedRepo}
      <div class="px-3 py-4 text-center text-tn-fg-muted text-xs">
        リポジトリを選択してください
      </div>
    {/if}
  </div>
</aside>
