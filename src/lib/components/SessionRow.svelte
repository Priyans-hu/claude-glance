<script lang="ts">
  import StatusIcon from "./StatusIcon.svelte";
  import { Bot, GitBranch } from "lucide-svelte";
  import type { Session } from "../types";
  import { formatRelativeTime } from "../time";
  import { copyText, resumeCommand } from "../clipboard";
  import { sessionStore } from "../stores.svelte";

  interface Props {
    session: Session;
  }

  const { session }: Props = $props();

  // Right-click menu state. One menu open at a time per row; `+page.svelte`
  // closes it via a window-level click handler.
  let menuOpen = $state(false);
  let menuX = $state(0);
  let menuY = $state(0);

  function onClick() {
    sessionStore.toggleExpanded(session.id);
  }

  function onContextMenu(event: MouseEvent) {
    event.preventDefault();
    menuX = event.clientX;
    menuY = event.clientY;
    menuOpen = true;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onClick();
    } else if (event.key === "Escape") {
      menuOpen = false;
    }
  }

  function closeMenu() {
    menuOpen = false;
  }

  async function copyId() {
    await copyText(session.id);
    closeMenu();
  }

  async function copyResume() {
    await copyText(resumeCommand(session.id));
    closeMenu();
  }
</script>

<svelte:window onclick={closeMenu} />

<div
  class="flex cursor-pointer items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:bg-cg-surface"
  role="button"
  tabindex="0"
  data-testid="session-row-{session.id}"
  onclick={onClick}
  oncontextmenu={onContextMenu}
  onkeydown={onKeydown}
>
  <StatusIcon status={session.status} size={14} />

  <div class="flex min-w-0 flex-1 items-center gap-2">
    <span class="max-w-[200px] shrink-0 truncate text-sm font-medium text-cg-text">
      {session.project}
    </span>

    {#if session.branch}
      <span class="flex shrink-0 items-center gap-1 font-mono text-xs text-cg-muted">
        <GitBranch size={11} strokeWidth={2} />
        <span class="max-w-[140px] truncate">{session.branch}</span>
      </span>
    {/if}

    <span class="min-w-0 flex-1 truncate text-sm text-cg-muted">
      {session.title}
    </span>
  </div>

  {#if session.subagentCount > 0}
    <span
      class="flex shrink-0 items-center gap-1 rounded bg-cg-surface px-1.5 py-0.5 text-xs text-cg-muted"
    >
      <Bot size={11} strokeWidth={2} />
      +{session.subagentCount}
    </span>
  {/if}

  <span class="w-16 shrink-0 text-right text-xs text-cg-muted tabular-nums">
    {formatRelativeTime(session.lastActivity)}
  </span>
</div>

{#if menuOpen}
  <div
    class="cg-context-menu"
    style:left="{menuX}px"
    style:top="{menuY}px"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      if (e.key === "Escape") closeMenu();
    }}
  >
    <button
      type="button"
      role="menuitem"
      class="cg-context-item"
      data-testid="session-row-copy-id-menu"
      onclick={copyId}
    >
      Copy session id
    </button>
    <button
      type="button"
      role="menuitem"
      class="cg-context-item"
      data-testid="session-row-copy-resume-menu"
      onclick={copyResume}
    >
      Copy resume command
    </button>
  </div>
{/if}

<style>
  .cg-context-menu {
    position: fixed;
    z-index: 50;
    min-width: 200px;
    overflow: hidden;
    border-radius: 6px;
    border: 1px solid var(--color-cg-border, #1f2937);
    background: var(--color-cg-surface, #111113);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    padding: 4px;
    display: flex;
    flex-direction: column;
  }

  .cg-context-item {
    background: transparent;
    border: none;
    color: var(--color-cg-text, #e5e7eb);
    font-size: 12px;
    text-align: left;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .cg-context-item:hover,
  .cg-context-item:focus-visible {
    background: rgba(255, 255, 255, 0.05);
    outline: none;
  }
</style>
