<script lang="ts">
  import { Eye } from "lucide-svelte";
</script>

<main class="flex min-h-screen flex-col gap-6 px-8 py-6">
  <header class="flex items-center gap-3">
    <Eye class="h-5 w-5 text-cg-waiting" strokeWidth={2.25} />
    <div>
      <h1 class="text-base font-semibold tracking-tight">claude-glance</h1>
      <p class="text-xs text-cg-muted">
        Read-only dashboard for active Claude Code sessions.
      </p>
    </div>
  </header>

  <section
    class="flex flex-1 items-center justify-center rounded-xl border border-dashed border-cg-border text-sm text-cg-muted"
  >
    UI lands in upcoming commits.
  </section>
</main>
