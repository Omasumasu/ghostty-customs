<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createWorktree, repoPath } from '$lib/stores/worktrees';
  import { get } from 'svelte/store';

  let {
    open,
    onclose,
  }: {
    open: boolean;
    onclose: () => void;
  } = $props();

  let branch = $state('');
  let baseBranch = $state('main');
  let launchClaude = $state(false);
  let initialPrompt = $state('');
  let creating = $state(false);
  let errorMsg = $state('');

  async function handleCreate() {
    if (!branch.trim()) {
      errorMsg = 'Branch name is required';
      return;
    }

    creating = true;
    errorMsg = '';

    try {
      const repo = get(repoPath);
      // Derive worktree path from repo path and branch name
      const safeBranch = branch.replace(/\//g, '-');
      const wtPath = `${repo}-${safeBranch}`;

      const wt = await createWorktree(repo, branch, wtPath, baseBranch);

      if (launchClaude) {
        await invoke('launch_claude', {
          worktreePath: wt.path,
          prompt: initialPrompt.trim() || null,
        });
      }

      // Reset form
      branch = '';
      baseBranch = 'main';
      launchClaude = false;
      initialPrompt = '';
      onclose();
    } catch (e) {
      errorMsg = String(e);
    } finally {
      creating = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
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
    <div class="bg-tn-bg-dark border border-tn-border rounded-lg w-[420px] shadow-xl">
      <div class="flex items-center justify-between px-4 py-3 border-b border-tn-border">
        <h3 class="text-sm font-medium text-tn-fg">New Worktree</h3>
        <button
          onclick={onclose}
          class="text-tn-fg-muted hover:text-tn-fg text-lg leading-none"
        >&times;</button>
      </div>

      <div class="p-4 space-y-3">
        <div>
          <label class="block text-xs text-tn-fg-muted mb-1" for="branch-name">Branch name</label>
          <input
            id="branch-name"
            type="text"
            bind:value={branch}
            placeholder="feature/my-feature"
            class="w-full px-3 py-1.5 text-sm bg-tn-bg border border-tn-border rounded text-tn-fg placeholder-tn-fg-muted focus:outline-none focus:border-tn-blue"
          />
        </div>

        <div>
          <label class="block text-xs text-tn-fg-muted mb-1" for="base-branch">Base branch</label>
          <input
            id="base-branch"
            type="text"
            bind:value={baseBranch}
            placeholder="main"
            class="w-full px-3 py-1.5 text-sm bg-tn-bg border border-tn-border rounded text-tn-fg placeholder-tn-fg-muted focus:outline-none focus:border-tn-blue"
          />
        </div>

        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            bind:checked={launchClaude}
            class="accent-tn-blue"
          />
          <span class="text-xs text-tn-fg-dim">Launch Claude Code</span>
        </label>

        {#if launchClaude}
          <div>
            <label class="block text-xs text-tn-fg-muted mb-1" for="initial-prompt">Initial prompt (optional)</label>
            <textarea
              id="initial-prompt"
              bind:value={initialPrompt}
              placeholder="Implement feature X..."
              rows="3"
              class="w-full px-3 py-1.5 text-sm bg-tn-bg border border-tn-border rounded text-tn-fg placeholder-tn-fg-muted focus:outline-none focus:border-tn-blue resize-none"
            ></textarea>
          </div>
        {/if}

        {#if errorMsg}
          <p class="text-xs text-tn-red">{errorMsg}</p>
        {/if}
      </div>

      <div class="flex justify-end gap-2 px-4 py-3 border-t border-tn-border">
        <button
          onclick={onclose}
          class="px-3 py-1.5 text-xs text-tn-fg-muted hover:text-tn-fg bg-tn-bg-alt border border-tn-border rounded transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleCreate}
          disabled={creating}
          class="px-3 py-1.5 text-xs text-tn-bg bg-tn-blue rounded hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {creating ? 'Creating...' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}
