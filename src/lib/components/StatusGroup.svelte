<script lang="ts">
  import { ChevronDown, ChevronRight } from "lucide-svelte";
  import StatusIcon from "./StatusIcon.svelte";
  import SessionRow from "./SessionRow.svelte";
  import SessionDetail from "./SessionDetail.svelte";
  import type { Session } from "../types";
  import type { SessionBucket } from "../grouping";
  import { BUCKET_LABEL } from "../grouping";
  import { sessionStore } from "../stores.svelte";

  interface Props {
    bucket: SessionBucket;
    sessions: Session[];
    collapsible?: boolean;
    expanded?: boolean;
    onToggle?: () => void;
  }

  const { bucket, sessions, collapsible = false, expanded = true, onToggle }: Props = $props();

  const showRows = $derived(!collapsible || expanded);
</script>

{#if sessions.length > 0}
  <section class="flex flex-col gap-1">
    {#if collapsible}
      <button
        type="button"
        class="flex items-center gap-2 rounded px-3 py-1 text-left transition-colors hover:bg-cg-surface"
        aria-expanded={expanded}
        data-testid="bucket-{bucket}-toggle"
        onclick={() => onToggle?.()}
      >
        <span class="text-cg-muted">
          {#if expanded}
            <ChevronDown size={12} strokeWidth={2.25} />
          {:else}
            <ChevronRight size={12} strokeWidth={2.25} />
          {/if}
        </span>
        <StatusIcon status={bucket} size={12} />
        <h2 class="text-xs font-semibold tracking-wider text-cg-muted uppercase">
          {BUCKET_LABEL[bucket]}
        </h2>
        <span class="text-xs text-cg-muted tabular-nums">{sessions.length}</span>
        {#if !expanded}
          <span class="text-xs text-cg-muted">click to show</span>
        {/if}
      </button>
    {:else}
      <header class="flex items-center gap-2 px-3 py-1">
        <StatusIcon status={bucket} size={12} />
        <h2 class="text-xs font-semibold tracking-wider text-cg-muted uppercase">
          {BUCKET_LABEL[bucket]}
        </h2>
        <span class="text-xs text-cg-muted tabular-nums">{sessions.length}</span>
      </header>
    {/if}
    {#if showRows}
      <ul class="flex flex-col">
        {#each sessions as session (session.id)}
          <li>
            <SessionRow {session} />
            {#if sessionStore.expandedSessionId === session.id}
              <SessionDetail {session} />
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}
