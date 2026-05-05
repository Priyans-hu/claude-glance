<script lang="ts">
  import { Activity, AlertCircle, BookOpen, Check, Pause, X } from "lucide-svelte";
  import type { SessionStatus } from "../types";

  interface Props {
    status: SessionStatus;
    size?: number;
    class?: string;
  }

  const { status, size = 16, class: className = "" }: Props = $props();

  const iconMap = {
    running: Activity,
    waiting: AlertCircle,
    plan: BookOpen,
    idle: Pause,
    done: Check,
    error: X,
  };

  const colorMap: Record<SessionStatus, string> = {
    running: "text-cg-running",
    waiting: "text-cg-waiting",
    plan: "text-cg-plan",
    idle: "text-cg-idle",
    done: "text-cg-done",
    error: "text-cg-error",
  };

  const Icon = $derived(iconMap[status]);
</script>

<Icon
  class="{colorMap[status]} {className}"
  {size}
  strokeWidth={2.25}
  aria-label="{status} status"
/>
