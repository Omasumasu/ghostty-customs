<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { worktrees, selectedWorktree, loadWorktrees, repoPath } from '$lib/stores/worktrees';
  import { sessions, initSessionListener, cleanupSessionListener, getSessionForWorktree } from '$lib/stores/sessions';
  import type { WorktreeInfo } from '$lib/types';
  import WorktreeList from '$lib/components/WorktreeList.svelte';
  import WorktreeDetail from '$lib/components/WorktreeDetail.svelte';
  import NewWorktreeDialog from '$lib/components/NewWorktreeDialog.svelte';
  import CleanupDialog from '$lib/components/CleanupDialog.svelte';

  let showNewDialog = $state(false);
  let showCleanupDialog = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    // Ask user to select a repository directory
    const selected = await open({
      directory: true,
      title: 'Select Git Repository',
    });

    if (selected && typeof selected === 'string') {
      await loadWorktrees(selected);
      await initSessionListener();

      // Auto-refresh every 30 seconds
      refreshInterval = setInterval(() => {
        const path = $repoPath;
        if (path) loadWorktrees(path);
      }, 30000);
    }
  });

  onDestroy(() => {
    cleanupSessionListener();
    if (refreshInterval) clearInterval(refreshInterval);
  });

  function handleSelect(wt: WorktreeInfo) {
    selectedWorktree.set(wt);
  }

  let currentSession = $derived(
    $selectedWorktree ? getSessionForWorktree($sessions, $selectedWorktree.path) : null
  );
</script>

<main class="flex h-screen bg-tn-bg">
  <WorktreeList
    worktrees={$worktrees}
    sessions={$sessions}
    selected={$selectedWorktree}
    onselect={handleSelect}
    onnewclick={() => showNewDialog = true}
    oncleanupclick={() => showCleanupDialog = true}
  />

  <WorktreeDetail
    worktree={$selectedWorktree}
    session={currentSession}
  />
</main>

<NewWorktreeDialog
  open={showNewDialog}
  onclose={() => showNewDialog = false}
/>

<CleanupDialog
  open={showCleanupDialog}
  onclose={() => showCleanupDialog = false}
/>
