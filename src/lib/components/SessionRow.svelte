<script lang="ts">
  import StatusIcon from "./StatusIcon.svelte";
  import { GitBranch, Bot } from "lucide-svelte";
  import type { Session } from "../types";
  import { formatRelativeTime } from "../time";

  interface Props {
    session: Session;
  }

  const { session }: Props = $props();
</script>

<div class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-cg-surface transition-colors">
  <StatusIcon status={session.status} size={14} />

  <div class="flex items-center gap-2 min-w-0 flex-1">
    <span class="text-sm font-medium text-cg-text shrink-0 truncate max-w-[200px]">
      {session.project}
    </span>

    {#if session.branch}
      <span class="flex items-center gap-1 text-xs text-cg-muted font-mono shrink-0">
        <GitBranch size={11} strokeWidth={2} />
        <span class="truncate max-w-[140px]">{session.branch}</span>
      </span>
    {/if}

    <span class="text-sm text-cg-muted truncate flex-1 min-w-0">
      {session.title}
    </span>
  </div>

  {#if session.subagentCount > 0}
    <span
      class="flex items-center gap-1 text-xs text-cg-muted shrink-0 px-1.5 py-0.5 rounded bg-cg-surface"
    >
      <Bot size={11} strokeWidth={2} />
      +{session.subagentCount}
    </span>
  {/if}

  <span class="text-xs text-cg-muted shrink-0 tabular-nums w-16 text-right">
    {formatRelativeTime(session.lastActivity)}
  </span>
</div>
