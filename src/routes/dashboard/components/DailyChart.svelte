<script lang="ts">
	import type { DailyChartItem } from '$lib/api';
	import type { AppLanguage } from '$lib/settings';
	import { onMount } from 'svelte';
	import * as echarts from 'echarts';

	let {
		items = [],
		loading = false,
		language = 'zh-CN'
	} = $props<{
		items?: DailyChartItem[];
		loading?: boolean;
		language?: AppLanguage;
	}>();

	function t(zh: string, en: string) {
		return language === 'zh-CN' ? zh : en;
	}

	let container = $state<HTMLDivElement | null>(null);
	let chart: echarts.ECharts | null = null;

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

	function getYAxisScale(chartItems: DailyChartItem[]) {
		const maxSeconds = Math.max(0, ...chartItems.map((item) => item.seconds));

		if (maxSeconds <= 0) {
			return {
				max: 3600,
				interval: 900
			};
		}

		if (maxSeconds < 60) {
			return {
				max: 60,
				interval: 15
			};
		}

		return {
			max: undefined,
			interval: undefined
		};
	}

	function areAnimationsEnabled() {
		return !document.documentElement.classList.contains('animations-disabled');
	}

	function buildOption(chartItems: DailyChartItem[]) {
		const isDark = document.documentElement.classList.contains('dark');
		const animationsEnabled = areAnimationsEnabled();
		const yAxisScale = getYAxisScale(chartItems);
		return {
			animation: animationsEnabled,
			animationDuration: animationsEnabled ? 500 : 0,
			animationDurationUpdate: animationsEnabled ? 300 : 0,
			tooltip: {
				trigger: 'axis',
				backgroundColor: isDark ? '#111827' : '#ffffff',
				borderColor: isDark ? '#1f2937' : '#d1d5db',
				textStyle: {
					color: isDark ? '#e5e7eb' : '#111827'
				},
				formatter(params: Array<{ axisValue: string; data: number }>) {
					const point = params[0];
					return `${point.axisValue}<br/>${formatAxisDuration(point.data)}`;
				}
			},
			grid: {
				left: 24,
				right: 16,
				top: 24,
				bottom: 24,
				containLabel: true
			},
			xAxis: {
				type: 'category',
				data: chartItems.map((item: DailyChartItem) => item.day.slice(5)),
				axisLine: {
					lineStyle: { color: isDark ? '#334155' : '#cbd5e1' }
				},
				axisLabel: {
					color: isDark ? '#94a3b8' : '#64748b'
				}
			},
			yAxis: {
				type: 'value',
				min: 0,
				max: yAxisScale.max,
				interval: yAxisScale.interval,
				splitLine: {
					lineStyle: { color: isDark ? '#1e293b' : '#e2e8f0' }
				},
				axisLabel: {
					color: isDark ? '#94a3b8' : '#64748b',
					formatter(value: number) {
						return formatAxisDuration(value);
					}
				}
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
						color: '#2563eb'
					},
					itemStyle: {
						color: '#0891b2'
					},
					areaStyle: {
						color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
							{ offset: 0, color: 'rgba(37, 99, 235, 0.22)' },
							{ offset: 1, color: 'rgba(8, 145, 178, 0.02)' }
						])
					}
				}
			]
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
			attributeFilter: ['class']
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

<article class="surface-card p-5">
	<div class="flex items-center justify-between gap-3">
		<div>
			<p class="eyebrow">{t('近 7 天游玩趋势', '7-Day Playtime Trend')}</p>
		</div>
	</div>

	{#if loading}
		<div
			class="mt-5 h-64 w-full rounded-xl bg-gray-50 p-4 sm:h-72 lg:h-80 xl:h-88 dark:bg-[#101522]"
		>
			<div class="flex h-full items-end gap-3">
				{#each [32, 48, 40, 62, 45, 70, 55] as h, index (index)}
					<div class="skeleton-shimmer flex-1 rounded-t-md" style={`height: ${h}%`}></div>
				{/each}
			</div>
		</div>
	{:else if items.length > 0}
		<div bind:this={container} class="mt-5 h-64 w-full sm:h-72 lg:h-80 xl:h-88"></div>
	{:else}
		<div class="empty-state mt-5 px-4 py-8">
			<p class="text-sm text-gray-500 dark:text-gray-400">
				{t('暂无趋势数据', 'No trend data')}
			</p>
		</div>
	{/if}
</article>
