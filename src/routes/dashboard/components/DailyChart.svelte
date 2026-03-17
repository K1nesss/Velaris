<script lang="ts">
  import type { DailyChartItem } from '$lib/api';
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  let { items = [], loading = false } = $props<{
    items?: DailyChartItem[];
    loading?: boolean;
  }>();

  let container = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function buildOption(chartItems: DailyChartItem[]) {
    const isDark = document.documentElement.classList.contains('dark');
    return {
      animationDuration: 500,
      tooltip: {
        trigger: 'axis',
        backgroundColor: isDark ? '#111827' : '#ffffff',
        borderColor: isDark ? '#1f2937' : '#d1d5db',
        textStyle: {
          color: isDark ? '#e5e7eb' : '#111827',
        },
        formatter(params: Array<{ axisValue: string; data: number }>) {
          const point = params[0];
          return `${point.axisValue}<br/>${Math.round(point.data / 60)} 分钟`;
        },
      },
      grid: {
        left: 24,
        right: 16,
        top: 24,
        bottom: 24,
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: chartItems.map((item: DailyChartItem) => item.day.slice(5)),
        axisLine: {
          lineStyle: { color: isDark ? '#334155' : '#cbd5e1' },
        },
        axisLabel: {
          color: isDark ? '#94a3b8' : '#64748b',
        },
      },
      yAxis: {
        type: 'value',
        splitLine: {
          lineStyle: { color: isDark ? '#1e293b' : '#e2e8f0' },
        },
        axisLabel: {
          color: isDark ? '#94a3b8' : '#64748b',
          formatter(value: number) {
            return `${Math.round(value / 3600)}h`;
          },
        },
      },
      series: [
        {
          data: chartItems.map((item: DailyChartItem) => item.seconds),
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 8,
          lineStyle: {
            width: 3,
            color: '#8b5cf6',
          },
          itemStyle: {
            color: '#a855f7',
          },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(168, 85, 247, 0.35)' },
              { offset: 1, color: 'rgba(168, 85, 247, 0.02)' },
            ]),
          },
        },
      ],
    };
  }

  function updateChart(chartItems: DailyChartItem[]) {
    if (!container) {
      return;
    }

    if (!chart) {
      chart = echarts.init(container);
    }

    chart.setOption(buildOption(chartItems));
    chart.resize();
  }

  $effect(() => {
    updateChart(items);
  });

  onMount(() => {
    updateChart(items);

    const handleResize = () => chart?.resize();
    const observer = new MutationObserver(() => updateChart(items));
    const resizeObserver = new ResizeObserver(() => chart?.resize());

    window.addEventListener('resize', handleResize);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });
    if (container) {
      resizeObserver.observe(container);
    }

    return () => {
      window.removeEventListener('resize', handleResize);
      observer.disconnect();
      resizeObserver.disconnect();
      chart?.dispose();
      chart = null;
    };
  });
</script>

<article class="rounded-2xl border border-purple-200/60 bg-white/90 p-5 shadow-sm backdrop-blur-sm dark:border-purple-900/60 dark:bg-[#151926]">
  <div class="flex items-center justify-between gap-3">
    <div>
      <p class="text-xs font-medium uppercase tracking-[0.24em] text-purple-600 dark:text-purple-400">折线图</p>
      <!-- <h3 class="mt-2 text-lg font-semibold text-gray-900 dark:text-gray-100">最近 7 天游玩趋势</h3> -->
    </div>
    {#if loading}
      <span class="rounded-full bg-purple-100 px-3 py-1 text-xs text-purple-700 dark:bg-purple-950/40 dark:text-purple-300">
        加载中
      </span>
    {/if}
  </div>

  {#if items.length > 0}
    <div bind:this={container} class="mt-5 h-64 w-full sm:h-72 lg:h-80 xl:h-88"></div>
  {:else}
    <div class="mt-5 rounded-xl border border-dashed border-gray-200 bg-gray-50 px-4 py-8 text-center dark:border-gray-800 dark:bg-[#101522]">
      <p class="text-sm text-gray-500 dark:text-gray-400">暂无折线图数据</p>
    </div>
  {/if}
</article>