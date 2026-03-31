<script lang="ts">
  import { chartable } from "$lib/charts";
  import { screenshot } from "$lib/stores.svelte";
  import { onDestroy } from "svelte";
  import type { EChartsOptions } from "$lib/charts";

  let {
    durationSec,
    dpsData,
    bossHpData,
    oncommit,
    class: className = ""
  }: {
    durationSec: number;
    dpsData: number[];
    bossHpData: { bars: number; p: number }[];
    oncommit: (w: { startSec: number; endSec: number } | null) => void;
    class?: string;
  } = $props();

  let containerEl: HTMLElement | null = $state(null);
  let destroyChart: (() => void) | null = null;
  let chartUpdate: ((opts: EChartsOptions) => void) | null = null;
  let echartsInst: any = null;
  let isActive = $state(false);
  let commitTimer: ReturnType<typeof setTimeout> | null = null;
  let initialized = false;

  function formatTime(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function labelFormatter(_: string, valueStr: string): string {
    const sec = Number(valueStr);
    if (isNaN(sec)) return "";
    const time = formatTime(sec);
    const idx = Math.round(sec / 5);
    if (bossHpData.length > 0 && idx < bossHpData.length) {
      const entry = bossHpData[idx];
      const pct = (entry.p * 100).toFixed(1);
      return entry.bars > 0 ? `${time}  ${entry.bars}x ${pct}%` : `${time}  ${pct}%`;
    }
    return time;
  }

  function buildOptions(): EChartsOptions {
    return {
      grid: { left: 0, right: 0, top: 0, bottom: 0, height: 0 },
      xAxis: {
        type: "category",
        show: false,
        data: dpsData.map((_, i) => i * 5)
      },
      yAxis: { type: "value", show: false },
      series: [
        {
          type: "line",
          data: dpsData,
          showSymbol: false,
          smooth: 0.1,
          lineStyle: { width: 0 },
          areaStyle: { opacity: 0 },
          silent: true
        }
      ],
      dataZoom: [
        {
          type: "slider",
          xAxisIndex: 0,
          fillerColor: "rgba(80,80,80,.5)",
          borderColor: "rgba(80,80,80,.5)",
          handleStyle: { color: "rgba(80,80,80,.5)" },
          moveHandleStyle: { color: "rgba(136,136,136)" },
          moveHandleSize: 0,
          height: 24,
          bottom: 4,
          left: 50,
          right: 50,
          showDetail: true,
          labelFormatter: labelFormatter,
          textStyle: {
            color: "#e5e5e5",
            fontSize: 11
          }
        }
      ],
      tooltip: { show: false }
    };
  }

  // Create chart once when container mounts
  $effect(() => {
    if (containerEl && !initialized) {
      initialized = true;
      const opts = buildOptions();
      const { destroy, echartsInstance, update } = chartable(containerEl, opts);
      destroyChart = destroy;
      chartUpdate = update;
      echartsInst = echartsInstance;

      echartsInstance.on("datazoom", (params: any) => {
        const start = params.start ?? params.batch?.[0]?.start ?? 0;
        const end = params.end ?? params.batch?.[0]?.end ?? 100;
        isActive = !(start <= 0.01 && end >= 99.99);
        if (commitTimer) clearTimeout(commitTimer);
        commitTimer = setTimeout(() => {
          const startSec = Math.round((start / 100) * durationSec);
          const endSec = Math.round((end / 100) * durationSec);
          oncommit(isActive ? { startSec, endSec } : null);
        }, 150);
      });
    }
  });

  // Update chart data without recreating
  $effect(() => {
    if (chartUpdate && dpsData.length > 0) {
      chartUpdate(buildOptions());
    }
  });

  onDestroy(() => {
    if (commitTimer) clearTimeout(commitTimer);
    if (destroyChart) destroyChart();
  });

  function reset() {
    echartsInst?.dispatchAction({ type: "dataZoom", start: 0, end: 100 });
    isActive = false;
    oncommit(null);
  }

</script>

{#if !screenshot.state}
  <div class="flex items-center gap-1 {className}">
    <div class="min-w-0 flex-1 h-[40px]" bind:this={containerEl}></div>
    <button
      onclick={reset}
      class="w-5 shrink-0 text-center text-xs text-neutral-500 transition hover:text-white {isActive
        ? ''
        : 'invisible'}"
      title="Reset window"
    >
      ✕
    </button>
  </div>
{/if}
