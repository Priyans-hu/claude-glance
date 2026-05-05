<script lang="ts">
  import { Activity, BookOpen, CircleAlert, Clock } from "lucide-svelte";
  import type { SessionStatus } from "../types";

  interface Props {
    status: SessionStatus;
    size?: number;
    class?: string;
  }

  const { status, size = 16, class: className = "" }: Props = $props();

  const iconMap = {
    working: Activity,
    waiting: CircleAlert,
    plan: BookOpen,
    idle: Clock,
  };

  const colorMap: Record<SessionStatus, string> = {
    working: "text-cg-working",
    waiting: "text-cg-waiting",
    plan: "text-cg-plan",
    idle: "text-cg-idle",
  };

  const Icon = $derived(iconMap[status]);
</script>

<Icon
  class="{colorMap[status]} {className}"
  {size}
  strokeWidth={2.25}
  aria-label="{status} status"
/>
