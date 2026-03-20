<script lang="ts">
  import { onMount } from 'svelte';
  import { getAnalyticsSnapshot, type AnalyticsSnapshot } from '$lib/api';
  import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';
  import * as echarts from 'echarts';

  const RANGE_OPTIONS = [1, 7, 30, 365] as const;

  const BAR_COLORS = ['#a855f7', '#f97316', '#22c55e', '#3b82f6', '#eab308', '#ef4444', '#0ea5e9', '#14b8a6'];

  let selectedDays = $state(7);
  let loading = $state(true);
  let refreshing = $state(false);
  let errorMessage = $state('');
  let snapshot = $state<AnalyticsSnapshot | null>(null);
  let language = $state<AppLanguage>('zh-CN');
  let analyticsRequestId = 0;

  const ANALYTICS_CACHE_KEY_PREFIX = 'analytics_snapshot_cache_v1';

    let topGamesContainer = $state<HTMLDivElement | null>(null);
    let distributionContainer = $state<HTMLDivElement | null>(null);
    let trendContainer = $state<HTMLDivElement | null>(null);

    let topGamesChart: echarts.ECharts | null = null;
    let distributionChart: echarts.ECharts | null = null;
    let trendChart: echarts.ECharts | null = null;

    function t(zh: string, en: string) {
      return language === 'zh-CN' ? zh : en;
    }

    function rangeLabel(days: (typeof RANGE_OPTIONS)[number]) {
      if (days === 1) return t('一天', '1 Day');
      if (days === 7) return t('一周', '7 Days');
      if (days === 30) return t('一月', '30 Days');
      return t('一年', '1 Year');
    }

    function formatNumber(value: number) {
      return new Intl.NumberFormat(language).format(value);
    }

    function formatDate(timestamp: number | null) {
      if (!timestamp) {
        return '-';
      }
      return new Intl.DateTimeFormat(language, {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      }).format(new Date(timestamp * 1000));
    }

    function avgSessionText(totalSeconds: number, sessions: number) {
      if (sessions <= 0) {
        return '0s';
      }
      const avg = Math.floor(totalSeconds / sessions);
      const h = Math.floor(avg / 3600);
      const m = Math.floor((avg % 3600) / 60);
      const s = avg % 60;
      if (h > 0) {
        return `${h}h ${m}m`;
      }
      if (m > 0) {
        return `${m}m ${s}s`;
      }
      return `${s}s`;
    }

    function trendClass(percentage: number) {
      if (percentage >= 12) {
        return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300';
      }
      if (percentage <= 4) {
        return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300';
      }
      return 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300';
    }

    function formatDurationCompact(seconds: number) {
      const value = Math.max(0, Math.round(seconds));
      const h = Math.floor(value / 3600);
      const m = Math.floor((value % 3600) / 60);
      const s = value % 60;

      if (h > 0) {
        return `${h}h ${m}m`;
      }
      if (m > 0) {
        return `${m}m`;
      }
      return `${s}s`;
    }

    function formatAxisDuration(value: number) {
      if (value >= 3600) {
        const hours = value / 3600;
        return Number.isInteger(hours) ? `${hours}h` : `${hours.toFixed(1)}h`;
      }
      if (value >= 60) {
        return `${Math.round(value / 60)}m`;
      }
      return `${Math.round(value)}s`;
    }

    function getAnalyticsCacheKey(days: number) {
      return `${ANALYTICS_CACHE_KEY_PREFIX}:${days}`;
    }

    function cacheAnalytics(days: number, data: AnalyticsSnapshot) {
      try {
        sessionStorage.setItem(getAnalyticsCacheKey(days), JSON.stringify(data));
      } catch {
        // Ignore cache failures.
      }
    }

    function readAnalyticsCache(days: number) {
      try {
        const raw = sessionStorage.getItem(getAnalyticsCacheKey(days));
        if (!raw) {
          return null;
        }
        return JSON.parse(raw) as AnalyticsSnapshot;
      } catch {
        return null;
      }
    }

    type TrendPoint = {
      label: string;
      seconds: number;
    };

    function getTrendPoints(data: NonNullable<AnalyticsSnapshot>): TrendPoint[] {
      if (selectedDays === 1 && data.hourly.length > 0) {
        return data.hourly.map((item) => ({
          label: item.label,
          seconds: item.seconds,
        }));
      }

      return data.daily.map((item) => ({
        label: item.day.slice(5),
        seconds: item.seconds,
      }));
    }

    function getHourlyInsights(data: NonNullable<AnalyticsSnapshot>) {
      const hourly = data.hourly;
      if (hourly.length === 0) {
        return {
          peakLabel: '--:--',
          peakValue: '0s',
          currentLabel: '--:--',
          currentValue: '0s',
          avgValue: '0s',
        };
      }

      let peak = hourly[0];
      let total = 0;
      for (const item of hourly) {
        total += item.seconds;
        if (item.seconds > peak.seconds) {
          peak = item;
        }
      }

      const currentHour = new Date().getHours();
      const current = hourly.find((item) => item.hour === currentHour) ?? hourly[hourly.length - 1];

      return {
        peakLabel: peak.label,
        peakValue: formatDurationCompact(peak.seconds),
        currentLabel: current.label,
        currentValue: formatDurationCompact(current.seconds),
        avgValue: formatDurationCompact(total / hourly.length),
      };
    }

    function buildTopGamesOption(data: NonNullable<AnalyticsSnapshot>['top_games']) {
      const isDark = document.documentElement.classList.contains('dark');
      const list = [...data].slice(0, 8).reverse();

      return {
        animationDuration: 450,
        tooltip: {
          trigger: 'axis',
          axisPointer: { type: 'shadow' },
          backgroundColor: isDark ? '#111827' : '#ffffff',
          borderColor: isDark ? '#1f2937' : '#d1d5db',
          textStyle: { color: isDark ? '#e5e7eb' : '#111827' },
          formatter(params: Array<{ name: string; data: number }>) {
            const point = params[0];
            const h = (point.data / 3600).toFixed(2);
            return `${point.name}<br/>${h} h`;
          },
        },
        grid: {
          left: 126,
          right: 26,
          top: 8,
          bottom: 8,
        },
        xAxis: {
          type: 'value',
          splitLine: { lineStyle: { color: isDark ? '#1e293b' : '#e2e8f0' } },
          axisLabel: {
            color: isDark ? '#94a3b8' : '#64748b',
            formatter(v: number) {
              return `${Math.round(v / 3600)}h`;
            },
          },
        },
        yAxis: {
          type: 'category',
          data: list.map((item) => item.name),
          axisLine: { show: false },
          axisTick: { show: false },
          axisLabel: {
            color: isDark ? '#cbd5e1' : '#475569',
            width: 116,
            overflow: 'truncate',
          },
        },
        series: [
          {
            type: 'bar',
            data: list.map((item, index) => ({
              value: item.seconds,
              itemStyle: {
                color: BAR_COLORS[(list.length - 1 - index) % BAR_COLORS.length],
                borderRadius: [0, 6, 6, 0],
              },
            })),
            barWidth: 18,
          },
        ],
      };
    }

    function buildDistributionOption(data: NonNullable<AnalyticsSnapshot>['top_games']) {
      const isDark = document.documentElement.classList.contains('dark');
      const pie = data.slice(0, 7).map((item, index) => ({
        name: item.name,
        value: item.seconds,
        itemStyle: { color: BAR_COLORS[index % BAR_COLORS.length] },
      }));

      return {
        animationDuration: 450,
        tooltip: {
          trigger: 'item',
          backgroundColor: isDark ? '#111827' : '#ffffff',
          borderColor: isDark ? '#1f2937' : '#d1d5db',
          textStyle: { color: isDark ? '#e5e7eb' : '#111827' },
          formatter(params: { name: string; value: number; percent: number }) {
            const h = (params.value / 3600).toFixed(2);
            return `${params.name}<br/>${h} h · ${params.percent}%`;
          },
        },
        legend: { show: false },
        series: [
          {
            type: 'pie',
            radius: ['58%', '78%'],
            center: ['50%', '50%'],
            itemStyle: {
              borderWidth: 3,
              borderColor: isDark ? '#111827' : '#ffffff',
            },
            label: { show: false },
            labelLine: { show: false },
            data: pie,
          },
        ],
      };
    }

    function buildTrendOption(points: TrendPoint[]) {
      const isDark = document.documentElement.classList.contains('dark');
      const isHourlyView = selectedDays === 1;
      const average = points.length > 0
        ? points.reduce((sum, point) => sum + point.seconds, 0) / points.length
        : 0;

      return {
        animationDuration: 450,
        tooltip: {
          trigger: 'axis',
          backgroundColor: isDark ? '#111827' : '#ffffff',
          borderColor: isDark ? '#1f2937' : '#d1d5db',
          textStyle: { color: isDark ? '#e5e7eb' : '#111827' },
          formatter(params: Array<{ axisValue: string; data: number; marker: string; seriesName?: string }>) {
            const title = params[0]?.axisValue ?? '';
            const lines = params.map((item) => `${item.marker}${item.seriesName ?? ''} ${formatDurationCompact(item.data)}`);
            return [title, ...lines].join('<br/>');
          },
        },
        grid: {
          left: 20,
          right: 10,
          top: 20,
          bottom: 24,
          containLabel: true,
        },
        xAxis: {
          type: 'category',
          data: points.map((item) => item.label),
          axisLabel: {
            color: isDark ? '#94a3b8' : '#64748b',
            formatter(value: string, index: number) {
              if (!isHourlyView) {
                return value;
              }
              return index % 2 === 0 ? value : '';
            },
          },
          axisLine: { lineStyle: { color: isDark ? '#334155' : '#cbd5e1' } },
          splitLine: isHourlyView
            ? { show: true, lineStyle: { color: isDark ? '#1f2937' : '#eef2ff', type: 'dashed' } }
            : { show: false },
        },
        yAxis: {
          type: 'value',
          axisLabel: {
            color: isDark ? '#94a3b8' : '#64748b',
            formatter(v: number) {
              return formatAxisDuration(v);
            },
          },
          splitLine: { lineStyle: { color: isDark ? '#1e293b' : '#e2e8f0' } },
        },
        series: [
          ...(isHourlyView
            ? [{
                name: t('每小时', 'Hourly'),
                type: 'bar',
                barWidth: 12,
                z: 1,
                itemStyle: {
                  borderRadius: [4, 4, 0, 0],
                  color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                    { offset: 0, color: '#38bdf8' },
                    { offset: 1, color: '#2563eb' },
                  ]),
                },
                data: points.map((item) => item.seconds),
              }]
            : []),
          {
            name: isHourlyView ? t('监控曲线', 'Monitoring Trend') : t('趋势', 'Trend'),
            type: 'line',
            smooth: !isHourlyView,
            symbol: 'circle',
            symbolSize: isHourlyView ? 4 : 6,
            z: 2,
            lineStyle: { color: '#3b82f6', width: 3 },
            itemStyle: { color: '#22d3ee' },
            areaStyle: {
              color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                { offset: 0, color: 'rgba(34, 211, 238, 0.35)' },
                { offset: 1, color: 'rgba(59, 130, 246, 0.03)' },
              ]),
            },
            markLine: isHourlyView
              ? {
                  symbol: 'none',
                  lineStyle: { type: 'dashed', color: isDark ? '#f59e0b' : '#d97706' },
                  label: {
                    color: isDark ? '#fcd34d' : '#92400e',
                    formatter: `${t('均值', 'Average')} ${formatDurationCompact(average)}`,
                  },
                  data: [{ yAxis: average }],
                }
              : undefined,
            data: points.map((item) => item.seconds),
          },
        ],
      };
    }

    function updateCharts() {
      if (!snapshot || loading) {
        return;
      }

      if (topGamesContainer) {
        if (!topGamesChart) {
          topGamesChart = echarts.init(topGamesContainer);
        }
        topGamesChart.setOption(buildTopGamesOption(snapshot.top_games));
        topGamesChart.resize();
      }

      if (distributionContainer) {
        if (!distributionChart) {
          distributionChart = echarts.init(distributionContainer);
        }
        distributionChart.setOption(buildDistributionOption(snapshot.top_games));
        distributionChart.resize();
      }

      if (trendContainer) {
        if (!trendChart) {
          trendChart = echarts.init(trendContainer);
        }
        trendChart.setOption(buildTrendOption(getTrendPoints(snapshot)));
        trendChart.resize();
      }
    }

    async function loadAnalytics(showLoading: boolean) {
      const requestId = ++analyticsRequestId;

      if (showLoading) {
        loading = true;
      } else {
        refreshing = true;
      }

      errorMessage = '';
      try {
        const days = selectedDays;
        const next = await getAnalyticsSnapshot(days, 8);
        if (requestId !== analyticsRequestId) {
          return;
        }

        snapshot = next;
        cacheAnalytics(days, next);
      } catch (error) {
        if (requestId !== analyticsRequestId) {
          return;
        }
        errorMessage = error instanceof Error ? error.message : String(error);
      } finally {
        if (requestId === analyticsRequestId) {
          loading = false;
          refreshing = false;
        }
      }
    }

    function selectRange(days: number) {
      if (selectedDays === days || loading || refreshing) {
        return;
      }
      selectedDays = days;

      const cached = readAnalyticsCache(days);
      if (cached) {
        snapshot = cached;
        loading = false;
      }

      void loadAnalytics(false);
    }

    onMount(() => {
      const settings = loadSettingsFromStorage();
      selectedDays = settings.analyticsDefaultRange;
      language = settings.language;

      const cached = readAnalyticsCache(selectedDays);
      if (cached) {
        snapshot = cached;
        loading = false;
      }

      void loadAnalytics(!cached);

      const onResize = () => {
        topGamesChart?.resize();
        distributionChart?.resize();
        trendChart?.resize();
      };

      const themeObserver = new MutationObserver(() => updateCharts());
      const resizeObserver = new ResizeObserver(onResize);

      window.addEventListener('resize', onResize);
      themeObserver.observe(document.documentElement, {
        attributes: true,
        attributeFilter: ['class'],
      });

      if (topGamesContainer) {
        resizeObserver.observe(topGamesContainer);
      }
      if (distributionContainer) {
        resizeObserver.observe(distributionContainer);
      }
      if (trendContainer) {
        resizeObserver.observe(trendContainer);
      }

      return () => {
        window.removeEventListener('resize', onResize);
        themeObserver.disconnect();
        resizeObserver.disconnect();
        topGamesChart?.dispose();
        distributionChart?.dispose();
        trendChart?.dispose();
        topGamesChart = null;
        distributionChart = null;
        trendChart = null;
      };
    });

    $effect(() => {
      snapshot;
      loading;
      updateCharts();
    });
  </script>

  <div class="h-full overflow-y-auto p-4 md:p-8">
    <div class="pb-8">
      <div class="mb-8 flex flex-col items-start justify-between gap-4 md:flex-row md:items-center">
        <div>
          <h1 class="text-2xl font-bold text-gray-800 dark:text-white">{t('数据分析总览', 'Analytics Overview')}</h1>
          <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('深度分析你的游玩习惯与统计数据', 'Analyze your play habits and statistics in depth')}</p>
        </div>

        <div class="inline-flex rounded-lg border border-gray-200 bg-white p-1 shadow-sm dark:border-gray-700 dark:bg-[#151926]">
          {#each RANGE_OPTIONS as days (days)}
            <button
              class={`rounded-md px-4 py-2 text-sm font-medium transition-colors ${
                selectedDays === days
                  ? 'border border-blue-200 bg-blue-50 text-blue-600 dark:border-blue-800/50 dark:bg-blue-900/30 dark:text-blue-400'
                  : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white'
              }`}
              onclick={() => selectRange(days)}
            >
              {rangeLabel(days)}
            </button>
          {/each}
        </div>
      </div>

      {#if errorMessage}
        <div class="mb-6 rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-950/20 dark:text-red-300">
          {errorMessage}
        </div>
      {/if}

      <div class="mb-8 grid grid-cols-1 gap-6 md:grid-cols-3">
        <div class="relative flex items-center justify-between overflow-hidden rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
          <div class="relative z-10">
            <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">{t('总游玩时长', 'Total Playtime')}</h3>
            {#if loading || !snapshot}
              <div class="h-8 w-24 rounded-md skeleton-shimmer"></div>
            {:else}
              <div class="text-2xl font-bold text-gray-900 dark:text-white">{snapshot.summary.total_formatted}</div>
              <div class="mt-1 inline-flex items-center rounded px-2 py-0.5 text-xs font-medium {snapshot.summary.active_days > 0 ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300' : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}">
                {t('活跃', 'Active')} {snapshot.summary.active_days} {t('天', 'days')}
              </div>
            {/if}
          </div>
          <div class="relative z-10 flex h-12 w-12 items-center justify-center rounded-full bg-purple-100 dark:bg-purple-900/30">
            <svg class="h-6 w-6 text-purple-600 dark:text-purple-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 3v18h18" />
              <path d="m7 14 4-4 3 3 5-5" />
            </svg>
          </div>
        </div>

        <div class="relative flex items-center justify-between overflow-hidden rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
          <div class="relative z-10">
            <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">{t('平均会话时长', 'Average Session')}</h3>
            {#if loading || !snapshot}
              <div class="h-8 w-24 rounded-md skeleton-shimmer"></div>
            {:else}
              <div class="text-2xl font-bold text-gray-900 dark:text-white">{snapshot.summary.average_session_formatted}</div>
              <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">{t('共', 'Total ')} {formatNumber(snapshot.summary.session_count)} {t('次会话', 'sessions')}</div>
            {/if}
          </div>
          <div class="relative z-10 flex h-12 w-12 items-center justify-center rounded-full bg-cyan-100 dark:bg-cyan-900/30">
            <svg class="h-6 w-6 text-cyan-600 dark:text-cyan-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="12" cy="12" r="9"></circle>
              <path d="M12 7v6l3 3"></path>
            </svg>
          </div>
        </div>

        <div class="relative flex items-center justify-between overflow-hidden rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
          <div class="relative z-10">
            <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">{t('最长单次会话', 'Longest Session')}</h3>
            {#if loading || !snapshot}
              <div class="h-8 w-24 rounded-md skeleton-shimmer"></div>
            {:else}
              <div class="text-2xl font-bold text-gray-900 dark:text-white">{snapshot.summary.longest_session_formatted}</div>
              <div class="mt-1 text-xs text-gray-500 dark:text-gray-400">{t('当前筛选区间内统计', 'Statistics within current range')}</div>
            {/if}
          </div>
          <div class="relative z-10 flex h-12 w-12 items-center justify-center rounded-full bg-blue-100 dark:bg-blue-900/30">
            <svg class="h-6 w-6 text-blue-600 dark:text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M3 12h18" />
              <path d="M12 3v18" />
              <path d="m16 8-4 4-4-4" />
            </svg>
          </div>
        </div>
      </div>

      <div class="mb-8 grid grid-cols-1 gap-6 lg:grid-cols-3">
        <div class="flex h-96 flex-col rounded-xl border border-gray-200 bg-white p-6 shadow-sm transition-colors duration-200 dark:border-gray-800 dark:bg-[#151926] lg:col-span-2">
          <div class="mb-6 flex items-center justify-between">
            <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">{t('游玩时长 Top 游戏', 'Top Games by Playtime')}</h2>
          </div>
          <div class="min-h-0 flex-1">
            {#if loading || !snapshot}
              <div class="h-full rounded-xl skeleton-shimmer"></div>
            {:else}
              <div bind:this={topGamesContainer} class="h-full w-full"></div>
            {/if}
          </div>
        </div>

        <div class="flex h-96 flex-col rounded-xl border border-gray-200 bg-white p-6 shadow-sm transition-colors duration-200 dark:border-gray-800 dark:bg-[#151926]">
          <h2 class="mb-6 text-lg font-semibold text-gray-800 dark:text-gray-100">{t('游玩时长分布', 'Playtime Distribution')}</h2>
          <div class="relative min-h-0 flex-1">
            {#if loading || !snapshot}
              <div class="h-full rounded-xl skeleton-shimmer"></div>
            {:else}
              <div bind:this={distributionContainer} class="h-full w-full"></div>
              <div class="pointer-events-none absolute inset-0 flex items-center justify-center">
                <span class="rounded bg-white/80 px-2 py-1 text-xs font-medium text-gray-500 dark:bg-[#151926]/80 dark:text-gray-400">{t('按时长', 'By Time')}</span>
              </div>
            {/if}
          </div>
          {#if snapshot}
            <div class="mt-4 grid grid-cols-2 gap-2 text-xs">
              {#each snapshot.top_games.slice(0, 4) as game, i (game.game_id)}
                <div class="flex items-center">
                  <span class="mr-2 h-2 w-2 rounded-full" style={`background-color: ${BAR_COLORS[i % BAR_COLORS.length]}`}></span>
                  <span class="truncate text-gray-600 dark:text-gray-400">{game.name}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <div class="mb-8 rounded-xl border border-gray-200 bg-white p-6 shadow-sm transition-colors duration-200 dark:border-gray-800 dark:bg-[#151926]">
        <h2 class="mb-4 text-lg font-semibold text-gray-800 dark:text-gray-100">{selectedDays === 1 ? t('小时趋势', 'Hourly Trend') : t('每日趋势', 'Daily Trend')}</h2>

        {#if selectedDays === 1 && snapshot}
          {@const hourlyInsights = getHourlyInsights(snapshot)}
          <div class="mb-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
            <div class="rounded-lg border border-cyan-200 bg-cyan-50/60 px-3 py-2 dark:border-cyan-900/50 dark:bg-cyan-950/20">
              <p class="text-[11px] uppercase tracking-wide text-cyan-700 dark:text-cyan-300">{t('峰值时段', 'Peak Slot')}</p>
              <p class="mt-1 text-sm font-semibold text-gray-900 dark:text-gray-100">{hourlyInsights.peakLabel} · {hourlyInsights.peakValue}</p>
            </div>
            <div class="rounded-lg border border-indigo-200 bg-indigo-50/60 px-3 py-2 dark:border-indigo-900/50 dark:bg-indigo-950/20">
              <p class="text-[11px] uppercase tracking-wide text-indigo-700 dark:text-indigo-300">{t('当前时段', 'Current Slot')}</p>
              <p class="mt-1 text-sm font-semibold text-gray-900 dark:text-gray-100">{hourlyInsights.currentLabel} · {hourlyInsights.currentValue}</p>
            </div>
            <div class="rounded-lg border border-amber-200 bg-amber-50/60 px-3 py-2 dark:border-amber-900/50 dark:bg-amber-950/20">
              <p class="text-[11px] uppercase tracking-wide text-amber-700 dark:text-amber-300">{t('小时均值', 'Hourly Avg')}</p>
              <p class="mt-1 text-sm font-semibold text-gray-900 dark:text-gray-100">{hourlyInsights.avgValue}</p>
            </div>
          </div>
        {/if}

        <div class="h-64">
          {#if loading || !snapshot}
            <div class="h-full rounded-xl skeleton-shimmer"></div>
          {:else}
            <div bind:this={trendContainer} class="h-full w-full"></div>
          {/if}
        </div>
      </div>

      <div class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm transition-colors duration-200 dark:border-gray-800 dark:bg-[#151926]">
        <div class="flex items-center justify-between border-b border-gray-200 p-6 dark:border-gray-800">
          <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-100">{t('详细游戏统计', 'Detailed Game Statistics')}</h2>
          <span class="text-sm text-gray-500 dark:text-gray-400">{t('按当前区间统计', 'Statistics in current range')}</span>
        </div>

        <div class="overflow-x-auto">
          <table class="w-full text-left text-sm">
            <thead class="bg-gray-50 text-xs font-medium uppercase text-gray-500 dark:bg-white/5 dark:text-gray-400">
              <tr>
                <th class="px-6 py-4">{t('游戏', 'Game')}</th>
                <th class="px-6 py-4">{t('总时长', 'Total Time')}</th>
                <th class="px-6 py-4">{t('会话数', 'Sessions')}</th>
                <th class="px-6 py-4">{t('平均会话', 'Avg. Session')}</th>
                <th class="px-6 py-4">{t('最后游玩', 'Last Played')}</th>
                <th class="px-6 py-4 text-right">{t('趋势', 'Trend')}</th>
              </tr>
            </thead>

            <tbody class="divide-y divide-gray-200 dark:divide-gray-800">
              {#if loading || !snapshot}
                {#each Array.from({ length: 4 }) as _, i (i)}
                  <tr>
                    <td class="px-6 py-4"><div class="h-5 w-40 rounded skeleton-shimmer"></div></td>
                    <td class="px-6 py-4"><div class="h-5 w-20 rounded skeleton-shimmer"></div></td>
                    <td class="px-6 py-4"><div class="h-5 w-12 rounded skeleton-shimmer"></div></td>
                    <td class="px-6 py-4"><div class="h-5 w-16 rounded skeleton-shimmer"></div></td>
                    <td class="px-6 py-4"><div class="h-5 w-24 rounded skeleton-shimmer"></div></td>
                    <td class="px-6 py-4"><div class="ml-auto h-5 w-12 rounded skeleton-shimmer"></div></td>
                  </tr>
                {/each}
              {:else if snapshot.top_games.length === 0}
                <tr>
                  <td colspan="6" class="px-6 py-10 text-center text-sm text-gray-500 dark:text-gray-400">{t('当前区间没有可展示的数据', 'No data available in current range')}</td>
                </tr>
              {:else}
                {#each snapshot.top_games as game, index (game.game_id)}
                  <tr class="transition-colors hover:bg-gray-50 dark:hover:bg-white/5">
                    <td class="px-6 py-4">
                      <div class="flex items-center">
                        <div
                          class="mr-3 flex h-8 w-8 items-center justify-center rounded font-bold text-white"
                          style={`background-color: ${BAR_COLORS[index % BAR_COLORS.length]}`}
                        >
                          {game.name.slice(0, 1).toUpperCase()}
                        </div>
                        <span class="font-medium text-gray-900 dark:text-white">{game.name}</span>
                      </div>
                    </td>
                    <td class="px-6 py-4 text-gray-600 dark:text-gray-300">{game.formatted}</td>
                    <td class="px-6 py-4 text-gray-600 dark:text-gray-300">{game.session_count}</td>
                    <td class="px-6 py-4 text-gray-600 dark:text-gray-300">{avgSessionText(game.seconds, game.session_count)}</td>
                    <td class="px-6 py-4 text-gray-600 dark:text-gray-300">{formatDate(game.last_played_at)}</td>
                    <td class="px-6 py-4 text-right">
                      <span class={`inline-flex items-center rounded px-2 py-0.5 text-xs font-medium ${trendClass(game.percentage)}`}>
                        {game.percentage.toFixed(1)}%
                      </span>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>