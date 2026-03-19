<script lang="ts">
  import type { DonutChartItem } from '$lib/api';
  import type { AppLanguage } from '$lib/settings';
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

  const LEGEND_SCROLL_THRESHOLD = 5;

  let { items = [], loading = false, language = 'zh-CN' } = $props<{
    items?: DonutChartItem[];
    loading?: boolean;
    language?: AppLanguage;
  }>();

  function t(zh: string, en: string) {
    return language === 'zh-CN' ? zh : en;
  }

  let container = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  function formatMinutes(value: number) {
    const hours = Math.floor(value / 3600);
    const minutes = Math.floor((value % 3600) / 60);
    return hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;
  }

  function buildOption() {
    const isDark = document.documentElement.classList.contains('dark');
    const useScrollableLegend = items.length > LEGEND_SCROLL_THRESHOLD;

    return {
      animationDuration: 500,
      tooltip: {
        trigger: 'item',
        backgroundColor: isDark ? '#111827' : '#ffffff',
        borderColor: isDark ? '#1f2937' : '#d1d5db',
        textStyle: {
          color: isDark ? '#e5e7eb' : '#111827',
        },
        formatter(params: { name: string; value: number; percent: number }) {
          return `${params.name}<br/>${formatMinutes(params.value)}<br/>${params.percent}%`;
        },
      },
      legend: {
        show: true,
        type: useScrollableLegend ? 'scroll' : 'plain',
        orient: 'vertical',
        left: 0,
        top: 8,
        bottom: 8,
        width: 144,
        itemWidth: 8,
        itemHeight: 8,
        itemGap: 10,
        pageButtonPosition: 'end',
        pageButtonGap: 8,
        pageButtonItemGap: 2,
        pageIconSize: 10,
        pageIconColor: '#ec4899',
        pageIconInactiveColor: isDark ? '#475569' : '#cbd5e1',
        pageIcons: {
          vertical: [
            'path://M512 320L224 640h576L512 320z',
            'path://M512 704l288-320H224l288 320z',
          ],
        },
        pageFormatter: () => '',
        pageTextStyle: {
          color: isDark ? '#cbd5e1' : '#475569',
          fontSize: 0,
        },
        textStyle: {
          color: isDark ? '#e2e8f0' : '#334155',
          fontSize: 11,
          lineHeight: 14,
          width: 118,
          overflow: 'truncate',
        },
      },
      series: [
        {
          type: 'pie',
          radius: ['60%', '95%'],
          center: ['70%', '50%'],
          avoidLabelOverlap: true,
          itemStyle: {
            borderRadius: 12,
            borderColor: isDark ? '#151926' : '#ffffff',
            borderWidth: 4,
          },
          label: {
            show: false,
          },
          labelLine: {
            show: false,
          },
          data: items.map((item: DonutChartItem) => ({
            value: item.seconds,
            name: item.name,
          })),
        },
      ],
      color: ['#06b6d4', '#8b5cf6', '#f59e0b', '#ef4444', '#22c55e', '#ec4899', '#14b8a6', '#3b82f6'],
    };
  }

  function updateChart() {
    if (!container) {
      return;
    }

    if (!chart) {
      chart = echarts.init(container);
    }

    chart.setOption(buildOption());
    chart.resize();
  }

  $effect(() => {
    updateChart();
  });

  onMount(() => {
    updateChart();

    const handleResize = () => chart?.resize();
    const observer = new MutationObserver(() => updateChart());
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

<article class="rounded-2xl border border-pink-200/60 bg-white p-5 shadow-sm dark:border-pink-900/60 dark:bg-[#151926]">
  <div class="flex items-center justify-between gap-3">
    <div>
      <p class="text-xs font-medium uppercase tracking-[0.24em] text-pink-600 dark:text-pink-400">{t('甜甜圈图', 'Donut Chart')}</p>
      <!-- <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">ECharts 原生图例滚动示例</p> -->
    </div>
  </div>

  <div class="mt-5 rounded-xl bg-white p-3 dark:bg-[#151926]">
    {#if loading}
      <div class="flex h-64 w-full items-center justify-center rounded-lg bg-gray-50 dark:bg-[#101522] sm:h-72 lg:h-80 xl:h-88">
        <div class="relative h-40 w-40 rounded-full skeleton-shimmer sm:h-44 sm:w-44">
          <div class="absolute inset-6 rounded-full bg-white dark:bg-[#101522]"></div>
        </div>
      </div>
    {:else}
      <div bind:this={container} class="h-64 w-full sm:h-72 lg:h-80 xl:h-88"></div>
    {/if}
  </div>
</article>