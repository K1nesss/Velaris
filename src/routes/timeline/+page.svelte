<script lang="ts">
	import { onMount } from 'svelte';
	import { fly } from 'svelte/transition';
	import { resolve } from '$app/paths';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { getTimelineSessions, syncSteamOwnedGameIcons, type TimelineSessionItem } from '$lib/api';
	import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';

	let pageSize = $state(80);
	let language = $state<AppLanguage>('zh-CN');

	type TimelineDayGroup = {
		key: string;
		label: string;
		totalSeconds: number;
		totalFormatted: string;
		sessions: TimelineSessionItem[];
	};

	type Accent = {
		dot: string;
		panel: string;
		chip: string;
	};

	type DurationRange = 'all' | 'lt30m' | '30to60m' | '1to2h' | 'gt2h';

	let loading = $state(true);
	let refreshing = $state(false);
	let loadingMore = $state(false);
	let errorMessage = $state('');
	let search = $state('');
	let debouncedSearch = $state('');
	let endedOnly = $state(false);
	let selectedGameId = $state('all');
	let durationRange = $state<DurationRange>('all');
	let showFilterPanel = $state(false);
	let openFilterSelect = $state<'game' | 'duration' | null>(null);
	let expandedSessionIds = $state<number[]>([]);
	let allSessions = $state<TimelineSessionItem[]>([]);
	let groups = $state<TimelineDayGroup[]>([]);
	let hasMore = $state(true);
	let filterPanelRef = $state<HTMLDivElement | null>(null);
	let filterButtonRef = $state<HTMLButtonElement | null>(null);
	let timelineRequestId = 0;

	const TIMELINE_CACHE_KEY = 'timeline_sessions_cache_v2_icons';

	function t(zh: string, en: string) {
		return language === 'zh-CN' ? zh : en;
	}

	const accents: Accent[] = [
		{
			dot: 'border-yellow-500 text-yellow-500',
			panel: 'bg-yellow-950/20',
			chip: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300'
		},
		{
			dot: 'border-red-500 text-red-500',
			panel: 'bg-red-950/20',
			chip: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300'
		},
		{
			dot: 'border-emerald-500 text-emerald-500',
			panel: 'bg-emerald-950/20',
			chip: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300'
		},
		{
			dot: 'border-indigo-500 text-indigo-500',
			panel: 'bg-indigo-950/20',
			chip: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300'
		},
		{
			dot: 'border-cyan-500 text-cyan-500',
			panel: 'bg-cyan-950/20',
			chip: 'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300'
		}
	];

	function getAccent(name: string) {
		const hash = Array.from(name).reduce((acc, char) => acc + char.charCodeAt(0), 0);
		return accents[hash % accents.length];
	}

	function getCoverSrc(path: string | null) {
		return path ? convertFileSrc(path) : null;
	}

	function cacheTimelineSessions(items: TimelineSessionItem[]) {
		try {
			sessionStorage.setItem(
				TIMELINE_CACHE_KEY,
				JSON.stringify({
					pageSize,
					items
				})
			);
		} catch {
			// Ignore cache failures.
		}
	}

	function hydrateTimelineFromCache() {
		try {
			const raw = sessionStorage.getItem(TIMELINE_CACHE_KEY);
			if (!raw) {
				return false;
			}

			const parsed = JSON.parse(raw) as {
				pageSize: number;
				items: TimelineSessionItem[];
			};

			if (parsed.pageSize !== pageSize || !Array.isArray(parsed.items)) {
				return false;
			}

			allSessions = parsed.items;
			hasMore = parsed.items.length === pageSize;
			buildGroups(parsed.items);
			loading = false;
			return true;
		} catch {
			return false;
		}
	}

	function formatDateTime(timestamp: number) {
		return new Intl.DateTimeFormat(language, {
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		}).format(new Date(timestamp * 1000));
	}

	function formatTimeRange(startTime: number, endTime: number | null) {
		const start = new Intl.DateTimeFormat('zh-CN', {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		}).format(new Date(startTime * 1000));

		if (!endTime) {
			return `${start} - ${t('进行中', 'Running')}`;
		}

		const end = new Intl.DateTimeFormat(language, {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false
		}).format(new Date(endTime * 1000));
		return `${start} - ${end}`;
	}

	function formatDuration(totalSeconds: number) {
		const seconds = Math.max(0, totalSeconds);
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		const remain = seconds % 60;

		if (hours > 0) {
			return `${hours}h ${minutes}m`;
		}
		if (minutes > 0) {
			return `${minutes}m ${remain}s`;
		}
		return `${remain}s`;
	}

	function formatGroupLabel(timestamp: number) {
		const target = new Date(timestamp * 1000);
		const today = new Date();
		const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000);

		const isSameDay =
			target.getFullYear() === today.getFullYear() &&
			target.getMonth() === today.getMonth() &&
			target.getDate() === today.getDate();

		if (isSameDay) {
			return t('今天', 'Today');
		}

		const isYesterday =
			target.getFullYear() === yesterday.getFullYear() &&
			target.getMonth() === yesterday.getMonth() &&
			target.getDate() === yesterday.getDate();

		if (isYesterday) {
			return t('昨天', 'Yesterday');
		}

		return new Intl.DateTimeFormat(language, {
			month: 'long',
			day: '2-digit',
			weekday: 'short'
		}).format(target);
	}

	function hasActiveFilters() {
		return endedOnly || selectedGameId !== 'all' || durationRange !== 'all';
	}

	function resetFilters() {
		endedOnly = false;
		selectedGameId = 'all';
		durationRange = 'all';
		openFilterSelect = null;
	}

	function toggleFilterSelect(target: 'game' | 'duration') {
		openFilterSelect = openFilterSelect === target ? null : target;
	}

	function selectGameFilter(value: string) {
		selectedGameId = value;
		openFilterSelect = null;
	}

	function selectDurationFilter(value: DurationRange) {
		durationRange = value;
		openFilterSelect = null;
	}

	function selectedGameLabel() {
		if (selectedGameId === 'all') {
			return t('全部游戏', 'All games');
		}

		return (
			getGameOptions().find((option) => String(option.id) === selectedGameId)?.name ??
			selectedGameId
		);
	}

	function durationRangeLabel(value: DurationRange) {
		if (value === 'lt30m') return t('小于 30 分钟', '< 30 minutes');
		if (value === '30to60m') return t('30 分钟 - 1 小时', '30 minutes - 1 hour');
		if (value === '1to2h') return t('1 小时 - 2 小时', '1 hour - 2 hours');
		if (value === 'gt2h') return t('大于等于 2 小时', '>= 2 hours');
		return t('全部时长', 'All durations');
	}

	function isSessionExpanded(sessionId: number) {
		return expandedSessionIds.includes(sessionId);
	}

	function toggleSession(sessionId: number) {
		if (expandedSessionIds.includes(sessionId)) {
			expandedSessionIds = expandedSessionIds.filter((id) => id !== sessionId);
		} else {
			expandedSessionIds = [...expandedSessionIds, sessionId];
		}
	}

	function getGameOptions() {
		const options: { id: number; name: string }[] = [];
		for (const session of allSessions) {
			if (!options.some((option) => option.id === session.game_id)) {
				options.push({
					id: session.game_id,
					name: session.game_name
				});
			}
		}

		return options.sort((a, b) => a.name.localeCompare(b.name, language));
	}

	function matchDurationRange(item: TimelineSessionItem) {
		const seconds = item.duration_seconds;

		if (durationRange === 'all') {
			return true;
		}
		if (durationRange === 'lt30m') {
			return seconds < 30 * 60;
		}
		if (durationRange === '30to60m') {
			return seconds >= 30 * 60 && seconds < 60 * 60;
		}
		if (durationRange === '1to2h') {
			return seconds >= 60 * 60 && seconds < 2 * 60 * 60;
		}
		return seconds >= 2 * 60 * 60;
	}

	function buildGroups(source: TimelineSessionItem[]) {
		const filtered = source.filter((item) => {
			if (endedOnly && item.is_active) {
				return false;
			}

			if (selectedGameId !== 'all' && item.game_id !== Number(selectedGameId)) {
				return false;
			}

			if (!matchDurationRange(item)) {
				return false;
			}

			const keyword = debouncedSearch;
			if (!keyword) {
				return true;
			}

			return (
				item.game_name.toLowerCase().includes(keyword) ||
				String(item.session_id).includes(keyword) ||
				String(item.appid ?? '').includes(keyword)
			);
		});

		const next: TimelineDayGroup[] = [];

		for (const item of filtered) {
			const date = new Date(item.start_time * 1000);
			const key = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;
			let group = next.find((entry) => entry.key === key);

			if (!group) {
				group = {
					key,
					label: formatGroupLabel(item.start_time),
					totalSeconds: 0,
					totalFormatted: '0s',
					sessions: []
				};
				next.push(group);
			}

			group.sessions.push(item);
			group.totalSeconds += item.duration_seconds;
		}

		for (const group of next) {
			group.totalFormatted = formatDuration(group.totalSeconds);
		}
		groups = next;
	}

	async function loadTimeline(showLoading: boolean) {
		const requestId = ++timelineRequestId;

		if (showLoading) {
			loading = true;
		} else {
			refreshing = true;
		}

		errorMessage = '';
		try {
			const data = await getTimelineSessions(pageSize, 0);
			if (requestId !== timelineRequestId) {
				return;
			}

			allSessions = data;
			hasMore = data.length === pageSize;
			buildGroups(allSessions);
			cacheTimelineSessions(data);
		} catch (error) {
			if (requestId !== timelineRequestId) {
				return;
			}
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (requestId === timelineRequestId) {
				loading = false;
				refreshing = false;
			}
		}
	}

	async function syncSteamIconsFromSettings() {
		const settings = loadSettingsFromStorage();

		if (!settings.apiKey || !settings.steam64Id) {
			return false;
		}

		try {
			const summary = await syncSteamOwnedGameIcons(settings.apiKey, settings.steam64Id);
			return summary.updated_games > 0;
		} catch (error) {
			console.warn('Failed to sync Steam game icons', error);
			return false;
		}
	}

	async function loadOlder() {
		if (!hasMore || loadingMore || loading || refreshing) {
			return;
		}

		loadingMore = true;
		try {
			const next = await getTimelineSessions(pageSize, allSessions.length);
			allSessions = [...allSessions, ...next];
			hasMore = next.length === pageSize;
			buildGroups(allSessions);
			cacheTimelineSessions(allSessions);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			loadingMore = false;
		}
	}

	onMount(() => {
		const settings = loadSettingsFromStorage();
		pageSize = settings.timelinePageSize;
		language = settings.language;

		const hydrated = hydrateTimelineFromCache();
		loadTimeline(!hydrated);
		syncSteamIconsFromSettings().then((hasUpdates) => {
			if (hasUpdates) {
				loadTimeline(false);
			}
		});

		const handlePointerDown = (event: PointerEvent) => {
			if (!showFilterPanel) {
				return;
			}

			const target = event.target as Node;
			if (filterPanelRef?.contains(target) || filterButtonRef?.contains(target)) {
				return;
			}

			showFilterPanel = false;
			openFilterSelect = null;
		};

		window.addEventListener('pointerdown', handlePointerDown);
		return () => {
			window.removeEventListener('pointerdown', handlePointerDown);
		};
	});

	$effect(() => {
		const rawSearch = search;
		const timer = window.setTimeout(() => {
			debouncedSearch = rawSearch.trim().toLowerCase();
		}, 140);

		return () => {
			window.clearTimeout(timer);
		};
	});

	$effect(() => {
		buildGroups(allSessions);
	});
</script>

<div class="app-page flex h-full flex-col">
	<div class="app-page-inner flex min-h-0 w-full flex-1 flex-col">
		<header class="page-header shrink-0">
			<div>
				<h1 class="page-title">{t('活动时间线', 'Activity Timeline')}</h1>
			</div>

			<div class="toolbar">
				<div class="relative w-full md:w-64">
					<input
						class="control-input w-full pr-4 pl-10"
						placeholder={t('搜索会话...', 'Search sessions...')}
						type="text"
						bind:value={search}
					/>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="pointer-events-none absolute top-2.5 left-3 h-5 w-5 text-gray-400"
						aria-hidden="true"
					>
						<circle cx="11" cy="11" r="8" />
						<path d="m21 21-4.3-4.3" />
					</svg>
				</div>

				<div class="relative">
					<button
						bind:this={filterButtonRef}
						class={`btn btn-icon ${
							hasActiveFilters()
								? 'border-blue-500 bg-blue-50 text-blue-600 dark:border-blue-500 dark:bg-blue-950/30 dark:text-blue-300'
								: 'btn-secondary'
						}`}
						title={t('筛选会话', 'Filter sessions')}
						onclick={() => {
							showFilterPanel = !showFilterPanel;
							if (!showFilterPanel) {
								openFilterSelect = null;
							}
						}}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="h-5 w-5"
							aria-hidden="true"
						>
							<path d="M3 5h18" />
							<path d="M7 12h10" />
							<path d="M10 19h4" />
						</svg>
					</button>

					{#if showFilterPanel}
						<div
							bind:this={filterPanelRef}
							class="surface-card absolute right-0 z-20 mt-2 w-72 origin-top-right p-4"
							transition:fly={{ y: -8, duration: 180 }}
						>
							<div class="space-y-4">
								<label
									class="flex cursor-pointer items-center justify-between gap-3 text-sm text-gray-700 dark:text-gray-200"
								>
									<span>{t('仅已结束', 'Ended only')}</span>
									<input type="checkbox" bind:checked={endedOnly} class="h-4 w-4" />
								</label>

								<div class="space-y-2">
									<label
										for="timeline-game-filter"
										class="text-xs font-medium tracking-wide text-gray-500 uppercase dark:text-gray-400"
										>{t('按游戏筛选', 'Filter by game')}</label
									>
									<div class="relative">
										<button
											id="timeline-game-filter"
											type="button"
											class="control-input flex w-full items-center justify-between gap-3 px-3 pr-3 text-left"
											aria-haspopup="listbox"
											aria-expanded={openFilterSelect === 'game'}
											onclick={() => toggleFilterSelect('game')}
										>
											<span class="min-w-0 truncate">{selectedGameLabel()}</span>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
												class={`h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200 ${
													openFilterSelect === 'game' ? 'rotate-180' : ''
												}`}
												aria-hidden="true"
											>
												<path d="m6 9 6 6 6-6" />
											</svg>
										</button>

										{#if openFilterSelect === 'game'}
											<div
												class="surface-card absolute right-0 left-0 z-30 mt-2 max-h-56 overflow-y-auto p-1"
												transition:fly={{ y: -6, duration: 160 }}
												role="listbox"
											>
												<button
													type="button"
													class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
														selectedGameId === 'all'
															? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
															: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
													}`}
													role="option"
													aria-selected={selectedGameId === 'all'}
													onclick={() => selectGameFilter('all')}
												>
													<span class="truncate">{t('全部游戏', 'All games')}</span>
												</button>
												{#each getGameOptions() as option (option.id)}
													<button
														type="button"
														class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
															selectedGameId === String(option.id)
																? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
																: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
														}`}
														role="option"
														aria-selected={selectedGameId === String(option.id)}
														onclick={() => selectGameFilter(String(option.id))}
													>
														<span class="truncate">{option.name}</span>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>

								<div class="space-y-2">
									<label
										for="timeline-duration-filter"
										class="text-xs font-medium tracking-wide text-gray-500 uppercase dark:text-gray-400"
										>{t('按时长区间', 'Filter by duration')}</label
									>
									<div class="relative">
										<button
											id="timeline-duration-filter"
											type="button"
											class="control-input flex w-full items-center justify-between gap-3 px-3 pr-3 text-left"
											aria-haspopup="listbox"
											aria-expanded={openFilterSelect === 'duration'}
											onclick={() => toggleFilterSelect('duration')}
										>
											<span class="min-w-0 truncate">{durationRangeLabel(durationRange)}</span>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
												class={`h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200 ${
													openFilterSelect === 'duration' ? 'rotate-180' : ''
												}`}
												aria-hidden="true"
											>
												<path d="m6 9 6 6 6-6" />
											</svg>
										</button>

										{#if openFilterSelect === 'duration'}
											<div
												class="surface-card absolute right-0 left-0 z-30 mt-2 p-1"
												transition:fly={{ y: -6, duration: 160 }}
												role="listbox"
											>
												{#each ['all', 'lt30m', '30to60m', '1to2h', 'gt2h'] as option (option)}
													<button
														type="button"
														class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
															durationRange === option
																? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
																: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
														}`}
														role="option"
														aria-selected={durationRange === option}
														onclick={() => selectDurationFilter(option as DurationRange)}
													>
														<span class="truncate"
															>{durationRangeLabel(option as DurationRange)}</span
														>
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>
								<div class="flex items-center justify-between gap-3 pt-1">
									<button
										class="rounded-lg border border-gray-300 px-3 py-1.5 text-xs text-gray-700 transition hover:bg-gray-100 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
										onclick={resetFilters}
									>
										{t('重置', 'Reset')}
									</button>
									<button
										class="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-blue-700"
										onclick={() => {
											showFilterPanel = false;
											openFilterSelect = null;
										}}
									>
										{t('应用', 'Apply')}
									</button>
								</div>
							</div>
						</div>
					{/if}
				</div>

				<button
					class="btn btn-primary relative w-24 disabled:cursor-not-allowed disabled:opacity-60"
					disabled={loading || refreshing}
					onclick={() => loadTimeline(false)}
				>
					<span class={`transition-opacity ${refreshing ? 'opacity-0' : 'opacity-100'}`}
						>{t('刷新', 'Refresh')}</span
					>
					<span
						class={`absolute inset-0 flex items-center justify-center transition-opacity ${refreshing ? 'opacity-100' : 'opacity-0'}`}
					>
						{t('刷新中...', 'Refreshing...')}
					</span>
				</button>
			</div>
		</header>

		<div class="no-scrollbar min-h-0 flex-1 overflow-y-auto pr-2">
			{#if errorMessage}
				<p
					class="status-message mb-4 border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300"
				>
					{t('加载失败：', 'Load failed: ')}{errorMessage}
				</p>
			{/if}

			{#if loading}
				<div class="mx-auto max-w-4xl space-y-6 pb-8">
					{#each [1, 2, 3] as row (row)}
						<article class="surface-card p-5">
							<div class="skeleton-shimmer h-5 w-48 rounded-md"></div>
							<div class="mt-4 space-y-3">
								{#each [1, 2] as item (item)}
									<div class="skeleton-shimmer h-24 rounded-xl"></div>
								{/each}
							</div>
						</article>
					{/each}
				</div>
			{:else if groups.length === 0}
				<article class="empty-state mx-auto max-w-4xl">
					<p class="text-sm text-gray-500 dark:text-gray-400">
						{t('暂无符合筛选条件的时间线数据', 'No timeline data for current filters')}
					</p>
				</article>
			{:else}
				<div class="mx-auto max-w-4xl space-y-10 pb-8">
					{#each groups as group (group.key)}
						<div class="relative">
							<div class="mb-4 flex items-center py-2">
								<span
									class="rounded-full border border-blue-200 bg-blue-100 px-3 py-1 text-xs font-bold tracking-wider text-blue-700 uppercase dark:border-blue-700/50 dark:bg-blue-900/30 dark:text-blue-300"
								>
									{group.label}
								</span>
								<div class="ml-4 h-px flex-1 bg-gray-300 dark:bg-gray-800"></div>
								<span class="ml-4 text-xs text-gray-500 dark:text-gray-500"
									>{group.totalFormatted} {t('总计', 'Total')}</span
								>
							</div>

							<div class="space-y-4">
								{#each group.sessions as session, index (session.session_id)}
									{@const accent = getAccent(session.game_name)}
									{@const coverSrc = getCoverSrc(session.icon_path)}
									<div class="relative pl-10">
										<div class="absolute top-0 bottom-0 left-0 flex flex-col items-center">
											<div
												class={`z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 bg-white dark:bg-[#151926] ${accent.dot}`}
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
													stroke-linecap="round"
													stroke-linejoin="round"
													class="h-4 w-4"
													aria-hidden="true"
												>
													<circle cx="12" cy="12" r="3"></circle>
													<path d="M12 2v3"></path>
													<path d="M12 19v3"></path>
													<path d="M4.93 4.93l2.12 2.12"></path>
													<path d="M16.95 16.95l2.12 2.12"></path>
													<path d="M2 12h3"></path>
													<path d="M19 12h3"></path>
													<path d="M4.93 19.07l2.12-2.12"></path>
													<path d="M16.95 7.05l2.12-2.12"></path>
												</svg>
											</div>
											{#if index < group.sessions.length - 1}
												<div class="mt-1 w-px flex-1 bg-gray-300 dark:bg-gray-700"></div>
											{/if}
										</div>

										<article
											class="surface-card overflow-hidden transition-all duration-200 hover:shadow-md dark:hover:border-gray-700"
										>
											<button
												type="button"
												class="flex w-full cursor-pointer items-center justify-between p-4 text-left select-none"
												aria-expanded={isSessionExpanded(session.session_id)}
												aria-controls={`timeline-session-${session.session_id}`}
												onclick={() => toggleSession(session.session_id)}
											>
												<div class="flex items-center space-x-4">
													<div
														class={`relative flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-lg ${accent.panel}`}
													>
														{#if coverSrc}
															<img
																src={coverSrc}
																alt=""
																class="h-8 w-8 rounded-md object-contain"
																loading="lazy"
															/>
														{:else}
															<span class="text-base font-bold text-white/90"
																>{session.game_name.slice(0, 1).toUpperCase()}</span
															>
															<div
																class={`absolute right-0 bottom-0 left-0 h-1 ${accent.chip.split(' ')[0]}`}
															></div>
														{/if}
													</div>
													<div>
														<h3 class="text-lg font-bold text-gray-900 dark:text-white">
															{session.game_name}
														</h3>
														<div
															class="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400"
														>
															<span
																class="rounded border border-gray-200 bg-gray-100 px-1.5 py-0.5 text-gray-600 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300"
																>Game #{session.game_id}</span
															>
															<span>•</span>
															<span class="font-mono"
																>{formatTimeRange(session.start_time, session.end_time)}</span
															>
														</div>
													</div>
												</div>

												<div class="flex items-center space-x-6">
													<div class="text-right">
														<div
															class="font-mono text-2xl font-bold text-gray-900 dark:text-gray-100"
														>
															{session.formatted}
														</div>
														{#if session.is_active}
															<div class="mt-0.5 text-xs font-medium text-emerald-500">
																{t('进行中', 'Running')}
															</div>
														{/if}
													</div>
													<svg
														xmlns="http://www.w3.org/2000/svg"
														viewBox="0 0 24 24"
														fill="none"
														stroke="currentColor"
														stroke-width="2"
														stroke-linecap="round"
														stroke-linejoin="round"
														class={`h-5 w-5 text-gray-400 transition-transform duration-250 ${
															isSessionExpanded(session.session_id) ? 'rotate-180' : ''
														}`}
														aria-hidden="true"
													>
														<path d="m6 9 6 6 6-6"></path>
													</svg>
												</div>
											</button>

											<div
												id={`timeline-session-${session.session_id}`}
												class={`grid transition-[grid-template-rows] duration-300 ease-out ${
													isSessionExpanded(session.session_id)
														? 'grid-rows-[1fr]'
														: 'grid-rows-[0fr]'
												}`}
											>
												<div class="overflow-hidden">
													<div
														class={`border-t border-gray-100 px-4 pt-0 pb-4 transition-all duration-300 ease-out dark:border-gray-800/50 ${
															isSessionExpanded(session.session_id)
																? 'translate-y-0 opacity-100'
																: '-translate-y-2 opacity-0'
														}`}
													>
														<div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
															<div
																class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]"
															>
																<span
																	class="mb-1 block text-xs tracking-wide text-gray-500 uppercase"
																	>{t('会话ID', 'Session ID')}</span
																>
																<div class="text-sm font-medium text-gray-800 dark:text-gray-300">
																	#{session.session_id}
																</div>
															</div>
															<div
																class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]"
															>
																<span
																	class="mb-1 block text-xs tracking-wide text-gray-500 uppercase"
																	>{t('开始时间', 'Started At')}</span
																>
																<div class="text-sm font-medium text-gray-800 dark:text-gray-300">
																	{formatDateTime(session.start_time)}
																</div>
															</div>
															<div
																class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]"
															>
																<span
																	class="mb-1 block text-xs tracking-wide text-gray-500 uppercase"
																	>{t('结束时间', 'Ended At')}</span
																>
																<div class="text-sm font-medium text-gray-800 dark:text-gray-300">
																	{session.end_time
																		? formatDateTime(session.end_time)
																		: t('仍在进行中', 'Still Running')}
																</div>
															</div>
														</div>

														<div class="mt-4 flex items-center justify-between">
															<a
																href={resolve(`/games/${session.game_id}`)}
																class="text-sm font-medium text-blue-600 transition-colors hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300"
															>
																{t('查看游戏详情', 'View game details')}
															</a>
															<span class={`rounded-full px-2.5 py-1 text-xs ${accent.chip}`}
																>AppID: {session.appid ?? '-'}</span
															>
														</div>
													</div>
												</div>
											</div>
										</article>
									</div>
								{/each}
							</div>
						</div>
					{/each}

					<div class="flex justify-center pt-2 pb-2">
						<button
							class="rounded-full border border-gray-200 bg-white px-6 py-2 text-sm font-medium text-gray-600 transition-colors hover:text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-800 dark:bg-[#151926] dark:text-gray-400 dark:hover:text-white"
							disabled={!hasMore || loadingMore}
							onclick={loadOlder}
						>
							{loadingMore
								? t('加载中...', 'Loading...')
								: hasMore
									? t('加载更早会话', 'Load Older Sessions')
									: t('没有更多会话', 'No More Sessions')}
						</button>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.no-scrollbar {
		scrollbar-width: none;
	}

	.no-scrollbar::-webkit-scrollbar {
		width: 0;
		height: 0;
	}
</style>
