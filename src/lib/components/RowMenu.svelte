<script lang="ts">
  import { MoreVertical } from "lucide-svelte";
  import { deleteSession, rescanSessions, stopSession } from "../api";
  import { sessionStore } from "../stores.svelte";

  interface Props {
    sessionId: string;
  }

  const { sessionId }: Props = $props();

  let open = $state(false);
  let busy = $state(false);
  let buttonEl: HTMLButtonElement | undefined = $state();

  function toggle(event: MouseEvent) {
    // The row itself toggles the detail strip on click; this menu must not
    // bubble its events up.
    event.stopPropagation();
    open = !open;
  }

  function close() {
    open = false;
  }

  function onWindowClick(event: MouseEvent) {
    if (!open) return;
    if (event.target instanceof Node && buttonEl?.contains(event.target)) return;
    close();
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  async function applySnapshot() {
    try {
      const sessions = await rescanSessions();
      sessionStore.sessions = sessions;
    } catch (err) {
      sessionStore.error = err instanceof Error ? err.message : String(err);
    }
  }

  async function onStop(event: MouseEvent) {
    event.stopPropagation();
    if (busy) return;
    busy = true;
    close();
    try {
      await stopSession(sessionId);
      await applySnapshot();
    } catch (err) {
      sessionStore.error = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }

  async function onDelete(event: MouseEvent) {
    event.stopPropagation();
    if (busy) return;
    // Native confirm() inside the Tauri webview keeps the dependency
    // surface small; the dialog plugin can be wired in later if we need
    // richer styling.
    const ok = window.confirm(
      "Delete this session?\n\nRemoves the JSONL transcript and the per-session subagents/tool-results directory.",
    );
    if (!ok) return;
    busy = true;
    close();
    try {
      await deleteSession(sessionId);
      await applySnapshot();
    } catch (err) {
      sessionStore.error = err instanceof Error ? err.message : String(err);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onWindowKeydown} />

<div class="cg-row-menu">
  <button
    bind:this={buttonEl}
    type="button"
    class="cg-row-menu-trigger"
    aria-haspopup="menu"
    aria-expanded={open}
    aria-label="Session actions"
    data-testid="row-menu-trigger-{sessionId}"
    disabled={busy}
    onclick={toggle}
  >
    <MoreVertical size={14} strokeWidth={2} />
  </button>
  {#if open}
    <div
      role="menu"
      tabindex="-1"
      class="cg-row-menu-popover"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          close();
        }
      }}
    >
      <button
        type="button"
        role="menuitem"
        class="cg-row-menu-item"
        data-testid="row-menu-stop-{sessionId}"
        onclick={onStop}
      >
        Stop session
      </button>
      <button
        type="button"
        role="menuitem"
        class="cg-row-menu-item cg-row-menu-item-danger"
        data-testid="row-menu-delete-{sessionId}"
        onclick={onDelete}
      >
        Delete session…
      </button>
    </div>
  {/if}
</div>

<style>
  .cg-row-menu {
    position: relative;
    display: inline-flex;
  }

  .cg-row-menu-trigger {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-cg-muted, #6b7280);
    cursor: pointer;
    /* Hidden until the row is hovered/focused; trigger remains accessible
       for keyboard users via Tab focusing the button itself. */
    opacity: 0;
    transition: opacity 0.12s ease-out;
  }

  /* Reveal on row hover/focus. SessionRow exposes `.cg-session-row` so we
     can target its hover state from this nested component. */
  :global(.cg-session-row:hover) .cg-row-menu-trigger,
  :global(.cg-session-row:focus-within) .cg-row-menu-trigger,
  .cg-row-menu-trigger:focus-visible,
  .cg-row-menu-trigger[aria-expanded="true"] {
    opacity: 1;
  }

  .cg-row-menu-trigger:hover,
  .cg-row-menu-trigger:focus-visible {
    background: rgba(255, 255, 255, 0.06);
    color: var(--color-cg-text, #e5e7eb);
    outline: none;
  }

  .cg-row-menu-trigger:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .cg-row-menu-popover {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 40;
    min-width: 180px;
    overflow: hidden;
    border-radius: 6px;
    border: 1px solid var(--color-cg-border, #1f2937);
    background: var(--color-cg-surface, #111113);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    padding: 4px;
    display: flex;
    flex-direction: column;
  }

  .cg-row-menu-item {
    background: transparent;
    border: none;
    color: var(--color-cg-text, #e5e7eb);
    font-size: 12px;
    text-align: left;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
  }

  .cg-row-menu-item:hover,
  .cg-row-menu-item:focus-visible {
    background: rgba(255, 255, 255, 0.05);
    outline: none;
  }

  .cg-row-menu-item-danger {
    color: var(--color-cg-error, #f87171);
  }
</style>
