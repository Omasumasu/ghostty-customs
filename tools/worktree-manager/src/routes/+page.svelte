<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { worktrees, selectedWorktree, loadWorktrees, repoPath } from '$lib/stores/worktrees';
  import { sessions, initSessionListener, cleanupSessionListener, getSessionForWorktree } from '$lib/stores/sessions';
  import { repositories, selectedRepo, loadRepositories } from '$lib/stores/repositories';
  import type { WorktreeInfo } from '$lib/types';
  import WorktreeList from '$lib/components/WorktreeList.svelte';
  import WorktreeDetail from '$lib/components/WorktreeDetail.svelte';
  import NewWorktreeDialog from '$lib/components/NewWorktreeDialog.svelte';
  import CleanupDialog from '$lib/components/CleanupDialog.svelte';

  let showNewDialog = $state(false);
  let showCleanupDialog = $state(false);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  async function switchRepo(path: string) {
    if (!path) {
      selectedRepo.set(null);
      worktrees.set([]);
      selectedWorktree.set(null);
      repoPath.set('');
      return;
    }
    selectedRepo.set(path);
    selectedWorktree.set(null);
    await loadWorktrees(path);
  }

  onMount(async () => {
    await loadRepositories();
    await initSessionListener();

    // Select the first saved repository if any
    const repos = $repositories;
    if (repos.length > 0) {
      await switchRepo(repos[0]);
    }

    // Auto-refresh every 30 seconds for the currently selected repo
    refreshInterval = setInterval(() => {
      const path = $repoPath;
      if (path) loadWorktrees(path);
    }, 30000);
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
    onrepochan={switchRepo}
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
