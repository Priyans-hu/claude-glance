<script lang="ts">
  import { Check, Copy, GitBranch } from "lucide-svelte";
  import type { Session } from "../types";
  import { copyText, resumeCommand } from "../clipboard";
  import { formatRelativeTime } from "../time";

  interface Props {
    session: Session;
  }

  const { session }: Props = $props();

  let copiedId = $state(false);
  let copiedCmd = $state(false);

  async function copyId() {
    if (await copyText(session.id)) {
      copiedId = true;
      setTimeout(() => (copiedId = false), 1200);
    }
  }

  async function copyResume() {
    if (await copyText(resumeCommand(session.id))) {
      copiedCmd = true;
      setTimeout(() => (copiedCmd = false), 1200);
    }
  }
</script>

<div
  class="ml-9 flex flex-col gap-1.5 rounded-md border border-cg-border/60 bg-cg-surface px-3 py-2 text-xs"
  data-testid="session-detail-{session.id}"
>
  <div class="flex items-center gap-2">
    <span class="w-16 text-cg-muted">id</span>
    <span class="font-mono text-cg-text">{session.id}</span>
    <button
      type="button"
      class="ml-1 flex items-center gap-1 rounded px-1.5 py-0.5 text-cg-muted transition-colors hover:bg-cg-bg hover:text-cg-text"
      data-testid="session-detail-copy-id-btn"
      onclick={copyId}
    >
      {#if copiedId}
        <Check size={11} strokeWidth={2.25} />
        copied
      {:else}
        <Copy size={11} strokeWidth={2} />
        copy
      {/if}
    </button>
  </div>

  <div class="flex items-center gap-2">
    <span class="w-16 text-cg-muted">cwd</span>
    <span class="truncate font-mono text-cg-text">{session.cwd || "—"}</span>
  </div>

  <div class="flex items-center gap-2">
    <span class="w-16 text-cg-muted">branch</span>
    {#if session.branch}
      <span class="flex items-center gap-1 font-mono text-cg-text">
        <GitBranch size={11} strokeWidth={2} />
        {session.branch}
      </span>
    {:else}
      <span class="text-cg-muted">—</span>
    {/if}
  </div>

  <div class="flex items-center gap-2">
    <span class="w-16 text-cg-muted">started</span>
    <span class="text-cg-text" title={session.lastActivity}>
      {formatRelativeTime(session.lastActivity)}
    </span>
  </div>

  <div class="mt-1 flex items-center gap-2 border-t border-cg-border/50 pt-2">
    <button
      type="button"
      class="flex items-center gap-1 rounded px-2 py-1 text-cg-muted transition-colors hover:bg-cg-bg hover:text-cg-text"
      data-testid="session-detail-copy-resume-btn"
      onclick={copyResume}
    >
      {#if copiedCmd}
        <Check size={11} strokeWidth={2.25} />
        copied
      {:else}
        <Copy size={11} strokeWidth={2} />
        copy resume command
      {/if}
    </button>
  </div>
</div>
