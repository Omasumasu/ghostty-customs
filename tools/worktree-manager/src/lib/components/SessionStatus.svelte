<script lang="ts">
  import type { SessionInfo } from '$lib/types';

  let { session }: { session: SessionInfo | null } = $props();

  let elapsed = $state('');

  $effect(() => {
    if (!session) {
      elapsed = '';
      return;
    }

    function updateElapsed() {
      if (!session) return;
      const lastSecs = parseInt(session.last_activity, 10);
      if (isNaN(lastSecs)) {
        elapsed = '';
        return;
      }
      const nowSecs = Math.floor(Date.now() / 1000);
      const diff = nowSecs - lastSecs;
      if (diff < 60) elapsed = `${diff}秒前`;
      else if (diff < 3600) elapsed = `${Math.floor(diff / 60)}分前`;
      else elapsed = `${Math.floor(diff / 3600)}時間前`;
    }

    updateElapsed();
    const interval = setInterval(updateElapsed, 5000);
    return () => clearInterval(interval);
  });

  function stateLabel(state: string): string {
    switch (state) {
      case 'Working': return '作業中';
      case 'Question': return '入力待ち';
      case 'Idle': return '待機中';
      case 'Merged': return 'マージ済み';
      default: return state;
    }
  }

  function stateColorClass(state: string): string {
    switch (state) {
      case 'Working': return 'text-tn-green';
      case 'Question': return 'text-tn-red';
      case 'Merged': return 'text-tn-purple';
      default: return 'text-tn-fg-muted';
    }
  }

  function stateBgClass(state: string): string {
    switch (state) {
      case 'Working': return 'bg-tn-green/10 border-tn-green/30';
      case 'Question': return 'bg-tn-red/10 border-tn-red/30';
      case 'Merged': return 'bg-tn-purple/10 border-tn-purple/30';
      default: return 'bg-tn-bg-alt border-tn-border';
    }
  }
</script>

<div class="rounded border {session ? stateBgClass(session.state) : 'bg-tn-bg-alt border-tn-border'} p-3">
  <div class="flex items-center justify-between mb-1">
    <span class="text-xs text-tn-fg-muted uppercase tracking-wider">Claude セッション</span>
    {#if elapsed}
      <span class="text-[10px] text-tn-fg-muted">{elapsed}</span>
    {/if}
  </div>

  {#if session}
    <div class="flex items-center gap-2">
      <span class="text-sm font-medium {stateColorClass(session.state)}">
        {stateLabel(session.state)}
      </span>
    </div>
    {#if session.state === 'Question' && session.question_text}
      <p class="mt-2 text-xs text-tn-yellow leading-relaxed">
        {session.question_text}
      </p>
    {/if}
  {:else}
    <span class="text-sm text-tn-fg-muted">アクティブなセッションなし</span>
  {/if}
</div>
