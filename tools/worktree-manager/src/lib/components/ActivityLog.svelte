<script lang="ts">
  import type { ActivityEntry } from '$lib/types';

  let { entries }: { entries: ActivityEntry[] } = $props();

  let recentEntries = $derived(entries.slice(-20).reverse());

  function typeBadge(actType: string): { label: string; classes: string } {
    switch (actType) {
      case 'ToolUse': return { label: 'ツール', classes: 'bg-tn-blue/20 text-tn-blue' };
      case 'Message': return { label: 'メッセージ', classes: 'bg-tn-green/20 text-tn-green' };
      case 'Error': return { label: 'エラー', classes: 'bg-tn-red/20 text-tn-red' };
      case 'Notification': return { label: '通知', classes: 'bg-tn-yellow/20 text-tn-yellow' };
      case 'Start': return { label: '開始', classes: 'bg-tn-purple/20 text-tn-purple' };
      case 'Stop': return { label: '停止', classes: 'bg-tn-fg-muted/20 text-tn-fg-muted' };
      default: return { label: actType, classes: 'bg-tn-bg-highlight text-tn-fg-muted' };
    }
  }

  function formatTimestamp(ts: string): string {
    const secs = parseInt(ts, 10);
    if (isNaN(secs)) return ts;
    const date = new Date(secs * 1000);
    return date.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }
</script>

<div class="flex flex-col">
  <div class="flex items-center justify-between mb-2">
    <span class="text-xs text-tn-fg-muted uppercase tracking-wider">アクティビティ</span>
    <span class="text-[10px] text-tn-fg-muted">{entries.length} 件</span>
  </div>

  <div class="space-y-1 max-h-[300px] overflow-y-auto">
    {#each recentEntries as entry (entry.timestamp + entry.summary)}
      {@const badge = typeBadge(entry.activity_type)}
      <div class="flex items-start gap-2 py-1 px-1 rounded hover:bg-tn-bg-alt">
        <span class="text-[10px] text-tn-fg-muted whitespace-nowrap mt-0.5 w-16 shrink-0">
          {formatTimestamp(entry.timestamp)}
        </span>
        <span class="text-[10px] px-1.5 py-0.5 rounded shrink-0 {badge.classes}">
          {badge.label}
        </span>
        <span class="text-xs text-tn-fg-dim leading-relaxed truncate">
          {entry.summary}
        </span>
      </div>
    {:else}
      <div class="text-xs text-tn-fg-muted py-2 text-center">アクティビティなし</div>
    {/each}
  </div>
</div>
