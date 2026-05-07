<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { CurrentPlayingGame } from '$lib/api';
	import type { AppLanguage } from '$lib/settings';

	let {
		currentPlaying = null,
		loading = false,
		language = 'zh-CN'
	} = $props<{
		currentPlaying?: CurrentPlayingGame | null;
		loading?: boolean;
		language?: AppLanguage;
	}>();

	function t(zh: string, en: string) {
		return language === 'zh-CN' ? zh : en;
	}

	function formatDateTime(timestamp: number) {
		return new Intl.DateTimeFormat(language, {
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		}).format(new Date(timestamp * 1000));
	}

	function formatDuration(totalSeconds: number) {
		const seconds = Math.max(0, Math.floor(totalSeconds));
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		const remainingSeconds = seconds % 60;
		const isZh = language === 'zh-CN';

		if (hours > 0) {
			return isZh ? `${hours}小时 ${minutes}分` : `${hours}h ${minutes}m`;
		}
		if (minutes > 0) {
			return isZh ? `${minutes}分 ${remainingSeconds}秒` : `${minutes}m ${remainingSeconds}s`;
		}
		return isZh ? `${remainingSeconds}秒` : `${remainingSeconds}s`;
	}

	let liveDurationFormatted = $state('0s');
	let durationTimerId: number | null = null;

	function refreshLiveDuration() {
		if (!currentPlaying) {
			liveDurationFormatted = '0s';
			return;
		}

		const nowSeconds = Math.floor(Date.now() / 1000);
		liveDurationFormatted = formatDuration(nowSeconds - currentPlaying.start_time);
	}

	$effect(() => {
		if (durationTimerId !== null) {
			window.clearInterval(durationTimerId);
			durationTimerId = null;
		}

		if (!currentPlaying) {
			liveDurationFormatted = '0s';
			return;
		}

		refreshLiveDuration();
		durationTimerId = window.setInterval(refreshLiveDuration, 1000);
	});

	onDestroy(() => {
		if (durationTimerId !== null) {
			window.clearInterval(durationTimerId);
			durationTimerId = null;
		}
	});
</script>

<article class="surface-card h-56 p-5 sm:h-56 lg:flex lg:h-56 lg:flex-col">
	<div class="flex items-center justify-between gap-3">
		<div>
			<p class="eyebrow">{t('正在游玩', 'Now Playing')}</p>
			<!-- <h3 class="mt-2 text-lg font-semibold text-gray-900 dark:text-gray-100">当前进行中的会话</h3> -->
		</div>
	</div>

	{#if loading}
		<div class="mt-5 flex min-h-0 flex-1 flex-col space-y-4">
			<div class="space-y-2">
				<div class="skeleton-shimmer h-7 w-40 rounded-md"></div>
				<div class="skeleton-shimmer h-4 w-24 rounded-md"></div>
			</div>

			<div class="grid min-h-0 flex-1 content-end gap-3 sm:grid-cols-2">
				<div class="subtle-panel p-4">
					<div class="skeleton-shimmer h-3 w-16 rounded-md"></div>
					<div class="skeleton-shimmer mt-2 h-6 w-24 rounded-md"></div>
				</div>
				<div class="subtle-panel p-4">
					<div class="skeleton-shimmer h-3 w-16 rounded-md"></div>
					<div class="skeleton-shimmer mt-2 h-5 w-32 rounded-md"></div>
				</div>
			</div>
		</div>
	{:else if currentPlaying}
		<div class="mt-5 flex min-h-0 flex-1 flex-col space-y-4">
			<div class="min-w-0 shrink-0">
				<div class="flex items-start justify-between gap-2">
					<p
						class="min-w-0 flex-1 truncate text-2xl leading-tight font-semibold text-gray-900 dark:text-gray-100"
					>
						{currentPlaying.game_name}
					</p>
					<p
						class="shrink-0 rounded-md bg-slate-100 px-2 py-1 text-xs text-gray-600 dark:bg-slate-800/80 dark:text-gray-300"
					>
						{t('会话', 'Session')} #{currentPlaying.session_id}
					</p>
				</div>
			</div>

			<div class="grid min-h-0 flex-1 gap-3 sm:grid-cols-2">
				<div class="subtle-panel min-w-0 p-4">
					<p class="text-xs tracking-wide text-amber-700 uppercase dark:text-amber-300">
						{t('已游玩', 'Played')}
					</p>
					<p class="metric-value mt-2 text-xl font-semibold">{liveDurationFormatted}</p>
				</div>
				<div class="subtle-panel min-w-0 p-4">
					<p class="text-xs tracking-wide text-gray-500 uppercase dark:text-gray-400">
						{t('开始时间', 'Start Time')}
					</p>
					<p
						class="mt-2 text-base leading-tight font-medium wrap-break-word text-gray-900 dark:text-gray-100"
					>
						{formatDateTime(currentPlaying.start_time)}
					</p>
				</div>
			</div>
		</div>
	{:else}
		<div class="mt-5 flex min-h-0 flex-1 flex-col">
			<div class="empty-state flex min-h-0 flex-1 items-center justify-center px-4 py-8">
				<p class="text-sm text-gray-500 dark:text-gray-400">
					{t('当前没有正在进行中的游戏会话', 'No game session is currently running')}
				</p>
			</div>
		</div>
	{/if}
</article>
