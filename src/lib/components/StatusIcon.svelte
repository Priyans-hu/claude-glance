<script lang="ts">
  import { Activity, Archive, BookOpen, CircleAlert, Clock } from "lucide-svelte";
  import type { SessionBucket } from "../grouping";

  interface Props {
    status: SessionBucket;
    size?: number;
    class?: string;
  }

  const { status, size = 16, class: className = "" }: Props = $props();

  const iconMap = {
    working: Activity,
    waiting: CircleAlert,
    plan: BookOpen,
    idle: Clock,
    recent: Archive,
  };

  const colorMap: Record<SessionBucket, string> = {
    working: "text-cg-working",
    waiting: "text-cg-waiting",
    plan: "text-cg-plan",
    idle: "text-cg-idle",
    recent: "text-cg-recent",
  };

  const Icon = $derived(iconMap[status]);
</script>

<Icon
  class="{colorMap[status]} {className}"
  {size}
  strokeWidth={2.25}
  aria-label="{status} status"
/>
