<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Eye } from "lucide-svelte";
  import StatusGroup from "$lib/components/StatusGroup.svelte";
  import StatusIcon from "$lib/components/StatusIcon.svelte";
  import { sessionStore } from "$lib/stores.svelte";
  import { STATUS_ORDER } from "$lib/grouping";
  import type { SessionStatus } from "$lib/types";

  onMount(() => {
    sessionStore.init();
  });

  onDestroy(() => {
    sessionStore.destroy();
  });

  // Header surfaces only the high-signal counts.
  const headerStatuses: SessionStatus[] = ["waiting", "running", "plan", "idle", "done"];
</script>

<main class="flex min-h-screen flex-col gap-4 px-6 py-5">
  <header class="flex items-center justify-between gap-4">
    <div class="flex items-center gap-3">
      <Eye class="h-5 w-5 text-cg-waiting" strokeWidth={2.25} />
      <div>
        <h1 class="text-base font-semibold tracking-tight">claude-glance</h1>
        <p class="text-xs text-cg-muted">
          {sessionStore.sessions.length} session{sessionStore.sessions.length === 1 ? "" : "s"}
        </p>
      </div>
    </div>

    <div class="flex items-center gap-3 text-xs text-cg-muted">
      {#each headerStatuses as status (status)}
        {#if sessionStore.counts[status] > 0}
          <span class="flex items-center gap-1 tabular-nums">
            <StatusIcon {status} size={12} />
            {sessionStore.counts[status]}
          </span>
        {/if}
      {/each}
    </div>
  </header>

  {#if sessionStore.error}
    <section
      class="flex flex-1 items-center justify-center rounded-xl border border-dashed border-cg-error text-sm text-cg-error"
    >
      {sessionStore.error}
    </section>
  {:else if sessionStore.loading}
    <section
      class="flex flex-1 items-center justify-center rounded-xl border border-dashed border-cg-border text-sm text-cg-muted"
    >
      Loading sessions…
    </section>
  {:else if sessionStore.sessions.length === 0}
    <section
      class="flex flex-1 flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-cg-border text-sm text-cg-muted"
    >
      <p>No Claude Code sessions found.</p>
      <p class="text-xs">Start a session in your terminal — it will appear here.</p>
    </section>
  {:else}
    <div class="flex flex-col gap-4">
      {#each STATUS_ORDER as status (status)}
        <StatusGroup {status} sessions={sessionStore.groupedByStatus[status]} />
      {/each}
    </div>
  {/if}
</main>
