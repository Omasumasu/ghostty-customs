<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { repoPath, loadWorktrees } from '$lib/stores/worktrees';
  import { get } from 'svelte/store';

  let {
    open,
    onclose,
  }: {
    open: boolean;
    onclose: () => void;
  } = $props();

  let cleaning = $state(false);
  let removed = $state<string[]>([]);
  let errorMsg = $state('');
  let done = $state(false);

  async function handleCleanup() {
    cleaning = true;
    errorMsg = '';
    removed = [];
    done = false;

    try {
      const repo = get(repoPath);
      const result = await invoke<string[]>('cleanup_merged', { repoPath: repo });
      removed = result;
      done = true;
      // Refresh worktree list
      await loadWorktrees(repo);
    } catch (e) {
      errorMsg = String(e);
    } finally {
      cleaning = false;
    }
  }

  function handleClose() {
    done = false;
    removed = [];
    errorMsg = '';
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') handleClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    onkeydown={handleKeydown}
  >
    <div class="bg-tn-bg-dark border border-tn-border rounded-lg w-[400px] shadow-xl">
      <div class="flex items-center justify-between px-4 py-3 border-b border-tn-border">
        <h3 class="text-sm font-medium text-tn-fg">マージ済み Worktree の整理</h3>
        <button
          onclick={handleClose}
          class="text-tn-fg-muted hover:text-tn-fg text-lg leading-none"
        >&times;</button>
      </div>

      <div class="p-4 space-y-3">
        {#if !done}
          <p class="text-xs text-tn-fg-dim leading-relaxed">
            メインブランチにマージ済みの Worktree を検出して削除します。
          </p>
        {:else}
          {#if removed.length > 0}
            <p class="text-xs text-tn-green mb-2">{removed.length} 件のマージ済み Worktree を削除しました:</p>
            <div class="space-y-1">
              {#each removed as branch}
                <div class="text-xs text-tn-fg-dim px-2 py-1 bg-tn-bg rounded">
                  {branch}
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-xs text-tn-fg-muted">マージ済みの Worktree はありません。</p>
          {/if}
        {/if}

        {#if errorMsg}
          <p class="text-xs text-tn-red">{errorMsg}</p>
        {/if}
      </div>

      <div class="flex justify-end gap-2 px-4 py-3 border-t border-tn-border">
        <button
          onclick={handleClose}
          class="px-3 py-1.5 text-xs text-tn-fg-muted hover:text-tn-fg bg-tn-bg-alt border border-tn-border rounded transition-colors"
        >
          {done ? '閉じる' : 'キャンセル'}
        </button>
        {#if !done}
          <button
            onclick={handleCleanup}
            disabled={cleaning}
            class="px-3 py-1.5 text-xs text-tn-bg bg-tn-yellow rounded hover:opacity-90 transition-opacity disabled:opacity-50"
          >
            {cleaning ? '整理中...' : '整理する'}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}
