<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Eye, RefreshCw } from "lucide-svelte";
  import StatusGroup from "$lib/components/StatusGroup.svelte";
  import StatusIcon from "$lib/components/StatusIcon.svelte";
  import { sessionStore } from "$lib/stores.svelte";
  import { BUCKET_ORDER } from "$lib/grouping";
  import { rescanSessions } from "$lib/api";
  import type { SessionStatus } from "$lib/types";

  onMount(() => {
    sessionStore.init();
  });

  onDestroy(() => {
    sessionStore.destroy();
  });

  // Header surfaces only the high-signal counts.
  const headerStatuses: SessionStatus[] = ["working", "waiting", "plan", "idle"];

  // Refresh button state. We keep the spinner visible for at least 400ms
  // so it doesn't flicker on a near-instant rescan, and surface the result
  // count for ~1.5s after.
  const SPINNER_MIN_MS = 400;
  const RESULT_VISIBLE_MS = 1500;
  let rescanning = $state(false);
  let lastCount: number | null = $state(null);
  let resultTimer: ReturnType<typeof setTimeout> | null = null;

  async function onRescan() {
    if (rescanning) return;
    rescanning = true;
    lastCount = null;
    if (resultTimer) {
      clearTimeout(resultTimer);
      resultTimer = null;
    }
    const startedAt = Date.now();
    try {
      const sessions = await rescanSessions();
      // Apply the snapshot directly — don't depend on the sessions_changed
      // event reaching the store before this promise resolves.
      sessionStore.sessions = sessions;
      const elapsed = Date.now() - startedAt;
      if (elapsed < SPINNER_MIN_MS) {
        await new Promise((r) => setTimeout(r, SPINNER_MIN_MS - elapsed));
      }
      lastCount = sessions.length;
      resultTimer = setTimeout(() => {
        lastCount = null;
        resultTimer = null;
      }, RESULT_VISIBLE_MS);
    } catch (err) {
      sessionStore.error = err instanceof Error ? err.message : String(err);
    } finally {
      rescanning = false;
    }
  }
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

      {#if lastCount !== null}
        <span class="text-xs text-cg-muted" data-testid="rescan-result">
          Re-scanned {lastCount} session{lastCount === 1 ? "" : "s"}
        </span>
      {/if}

      <button
        type="button"
        class="flex items-center gap-1 rounded p-1 text-cg-muted transition-colors hover:bg-cg-surface hover:text-cg-text disabled:opacity-50"
        title="Re-scan sessions"
        aria-label="Re-scan sessions"
        data-testid="header-rescan-btn"
        disabled={rescanning}
        onclick={onRescan}
      >
        <span class={rescanning ? "cg-spin" : ""}>
          <RefreshCw size={14} strokeWidth={2} />
        </span>
      </button>
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
      {#each BUCKET_ORDER as bucket (bucket)}
        <StatusGroup
          {bucket}
          sessions={sessionStore.bucketed[bucket]}
          collapsible={bucket === "recent"}
          expanded={sessionStore.showRecent}
          onToggle={bucket === "recent" ? () => sessionStore.toggleShowRecent() : undefined}
        />
      {/each}
    </div>
  {/if}
</main>

<style>
  .cg-spin {
    display: inline-flex;
    animation: cg-spin 0.9s linear infinite;
  }
  @keyframes cg-spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
