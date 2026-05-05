<script lang="ts">
  import StatusIcon from "./StatusIcon.svelte";
  import SessionRow from "./SessionRow.svelte";
  import type { Session, SessionStatus } from "../types";
  import { STATUS_LABEL } from "../grouping";

  interface Props {
    status: SessionStatus;
    sessions: Session[];
  }

  const { status, sessions }: Props = $props();
</script>

{#if sessions.length > 0}
  <section class="flex flex-col gap-1">
    <header class="flex items-center gap-2 px-3 py-1">
      <StatusIcon {status} size={12} />
      <h2 class="text-xs font-semibold uppercase tracking-wider text-cg-muted">
        {STATUS_LABEL[status]}
      </h2>
      <span class="text-xs text-cg-muted tabular-nums">{sessions.length}</span>
    </header>
    <ul class="flex flex-col">
      {#each sessions as session (session.id)}
        <li><SessionRow {session} /></li>
      {/each}
    </ul>
  </section>
{/if}
