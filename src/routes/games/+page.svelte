<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { resolve } from '$app/paths';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		createManualGame,
		getGamesList,
		syncSteamOwnedGameIcons,
		type GameListItem
	} from '$lib/api';
	import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';

	let loading = $state(true);
	let refreshing = $state(false);
	let search = $state('');
	let installedOnly = $state(false);
	let errorMessage = $state('');
	let games = $state<GameListItem[]>([]);
	let language = $state<AppLanguage>('zh-CN');
	let gamesRequestId = 0;
	let autoFiltersReady = false;
	let lastRequestedGamesKey = '';
	let pendingRestoreScrollTop: number | null = null;
	let restoringViewPosition = $state(false);
	let addGameDialogOpen = $state(false);
	let addGameSaving = $state(false);
	let addGameError = $state('');
	let manualGameForm = $state({
		name: '',
		executable_path: '',
		icon_path: '',
		cover_path: '',
		hero_path: ''
	});

	const GAMES_CACHE_KEY = 'games_list_cache_v2_covers';
	const GAMES_VIEW_STATE_KEY = 'games_view_state_v1';
	const GAMES_RESTORE_FROM_DETAIL_KEY = 'games_restore_from_detail_v1';
	const AUTO_FILTER_DEBOUNCE_MS = 240;

	function t(zh: string, en: string) {
		return language === 'zh-CN' ? zh : en;
	}

	function formatDateTime(timestamp: number | null) {
		if (!timestamp) {
			return '-';
		}
		return new Intl.DateTimeFormat(language, {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		}).format(new Date(timestamp * 1000));
	}

	function buildGamesCacheKey() {
		return `${search.trim().toLowerCase()}|${installedOnly ? '1' : '0'}`;
	}

	function getScrollContainer() {
		return document.querySelector<HTMLElement>('main.material-main');
	}

	function saveGamesViewState() {
		try {
			sessionStorage.setItem(GAMES_RESTORE_FROM_DETAIL_KEY, '1');
			sessionStorage.setItem(
				GAMES_VIEW_STATE_KEY,
				JSON.stringify({
					search,
					installedOnly,
					scrollTop: getScrollContainer()?.scrollTop ?? 0
				})
			);
		} catch {
			// Ignore view state failures.
		}
	}

	function restoreGamesViewState() {
		try {
			if (sessionStorage.getItem(GAMES_RESTORE_FROM_DETAIL_KEY) !== '1') {
				sessionStorage.removeItem(GAMES_VIEW_STATE_KEY);
				return;
			}
			sessionStorage.removeItem(GAMES_RESTORE_FROM_DETAIL_KEY);

			const raw = sessionStorage.getItem(GAMES_VIEW_STATE_KEY);
			if (!raw) {
				return;
			}

			const parsed = JSON.parse(raw) as {
				search?: string;
				installedOnly?: boolean;
				scrollTop?: number;
			};

			search = typeof parsed.search === 'string' ? parsed.search : '';
			installedOnly = typeof parsed.installedOnly === 'boolean' ? parsed.installedOnly : false;
			pendingRestoreScrollTop =
				typeof parsed.scrollTop === 'number' && Number.isFinite(parsed.scrollTop)
					? parsed.scrollTop
					: null;
			restoringViewPosition = pendingRestoreScrollTop !== null;
		} catch {
			// Ignore invalid view state.
		}
	}

	async function restoreGamesScrollPosition() {
		if (pendingRestoreScrollTop === null) {
			return;
		}

		const scrollTop = pendingRestoreScrollTop;
		pendingRestoreScrollTop = null;
		await tick();
		window.requestAnimationFrame(() => {
			window.requestAnimationFrame(() => {
				getScrollContainer()?.scrollTo({ top: scrollTop });
				restoringViewPosition = false;
			});
		});
	}

	function hydrateGamesFromCache() {
		try {
			const raw = sessionStorage.getItem(GAMES_CACHE_KEY);
			if (!raw) {
				return false;
			}

			const parsed = JSON.parse(raw) as {
				key: string;
				items: GameListItem[];
			};

			if (parsed.key !== buildGamesCacheKey() || !Array.isArray(parsed.items)) {
				return false;
			}

			games = parsed.items;
			loading = false;
			return true;
		} catch {
			return false;
		}
	}

	function cacheGames(items: GameListItem[]) {
		try {
			sessionStorage.setItem(
				GAMES_CACHE_KEY,
				JSON.stringify({
					key: buildGamesCacheKey(),
					items
				})
			);
		} catch {
			// Ignore cache failures.
		}
	}

	function getVisibleGames() {
		const keyword = search.trim().toLowerCase();

		return games.filter((game) => {
			if (installedOnly && !game.is_installed) {
				return false;
			}

			if (!keyword) {
				return true;
			}

			return game.name.toLowerCase().includes(keyword);
		});
	}

	function getAssetSrc(path: string | null) {
		return path ? convertFileSrc(path) : null;
	}

	function resetManualGameForm() {
		manualGameForm = {
			name: '',
			executable_path: '',
			icon_path: '',
			cover_path: '',
			hero_path: ''
		};
		addGameError = '';
	}

	function openAddGameDialog() {
		resetManualGameForm();
		addGameDialogOpen = true;
	}

	function closeAddGameDialog() {
		if (addGameSaving) {
			return;
		}
		addGameDialogOpen = false;
		addGameError = '';
	}

	function selectedPath(value: string | string[] | null) {
		if (typeof value === 'string') {
			return value;
		}
		if (Array.isArray(value) && typeof value[0] === 'string') {
			return value[0];
		}
		return null;
	}

	function inferGameNameFromPath(path: string) {
		const filename = path.split(/[\\/]/).pop() ?? '';
		return filename
			.replace(/\.[^.]+$/, '')
			.replace(/[_-]+/g, ' ')
			.trim();
	}

	async function chooseExecutable() {
		const path = selectedPath(
			await open({
				multiple: false,
				filters: [{ name: 'Executable', extensions: ['exe'] }]
			})
		);

		if (!path) {
			return;
		}

		manualGameForm.executable_path = path;
		if (!manualGameForm.name.trim()) {
			manualGameForm.name = inferGameNameFromPath(path);
		}
	}

	async function chooseImage(target: 'icon_path' | 'cover_path' | 'hero_path') {
		const extensions =
			target === 'icon_path'
				? ['ico', 'png', 'jpg', 'jpeg', 'webp']
				: ['png', 'jpg', 'jpeg', 'webp'];
		const path = selectedPath(
			await open({
				multiple: false,
				filters: [{ name: 'Images', extensions }]
			})
		);

		if (path) {
			manualGameForm[target] = path;
		}
	}

	async function submitManualGame() {
		if (addGameSaving) {
			return;
		}

		addGameError = '';
		const name = manualGameForm.name.trim();
		const executablePath = manualGameForm.executable_path.trim();

		if (!name) {
			addGameError = t('请填写游戏名称', 'Please enter a game name');
			return;
		}
		if (!executablePath) {
			addGameError = t('请选择游戏 exe 文件', 'Please choose the game executable');
			return;
		}

		addGameSaving = true;
		try {
			await createManualGame({
				name,
				executable_path: executablePath,
				icon_path: manualGameForm.icon_path.trim() || null,
				cover_path: manualGameForm.cover_path.trim() || null,
				hero_path: manualGameForm.hero_path.trim() || null
			});
			try {
				sessionStorage.removeItem(GAMES_CACHE_KEY);
			} catch {
				// Ignore cache cleanup failures.
			}
			addGameDialogOpen = false;
			resetManualGameForm();
			await loadGames(false);
		} catch (error) {
			addGameError = error instanceof Error ? error.message : String(error);
		} finally {
			addGameSaving = false;
		}
	}

	async function syncSteamMediaFromSettings() {
		const settings = loadSettingsFromStorage();

		if (!settings.apiKey || !settings.steam64Id) {
			return false;
		}

		try {
			const summary = await syncSteamOwnedGameIcons(settings.apiKey, settings.steam64Id);
			return summary.updated_games > 0;
		} catch (error) {
			console.warn('Failed to sync Steam game media', error);
			return false;
		}
	}

	async function loadGames(showLoading: boolean) {
		const requestId = ++gamesRequestId;
		const requestKey = buildGamesCacheKey();
		lastRequestedGamesKey = requestKey;

		if (showLoading) {
			loading = true;
		} else {
			refreshing = true;
		}

		errorMessage = '';
		try {
			const next = await getGamesList(search.trim(), installedOnly, 500);
			if (requestId !== gamesRequestId) {
				return;
			}

			games = next;
			cacheGames(next);
		} catch (error) {
			if (requestId !== gamesRequestId) {
				return;
			}
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (requestId === gamesRequestId) {
				loading = false;
				refreshing = false;
				restoreGamesScrollPosition();
			}
		}
	}

	onMount(() => {
		const settings = loadSettingsFromStorage();
		language = settings.language;
		restoreGamesViewState();

		const hydrated = hydrateGamesFromCache();
		if (hydrated) {
			restoreGamesScrollPosition();
		}
		loadGames(!hydrated);
		syncSteamMediaFromSettings().then((hasUpdates) => {
			if (hasUpdates) {
				loadGames(false);
			}
		});
		autoFiltersReady = true;
	});

	$effect(() => {
		const filterKey = buildGamesCacheKey();
		if (!autoFiltersReady || filterKey === lastRequestedGamesKey) {
			return;
		}

		const timer = window.setTimeout(() => {
			if (filterKey !== lastRequestedGamesKey) {
				void loadGames(false);
			}
		}, AUTO_FILTER_DEBOUNCE_MS);

		return () => {
			window.clearTimeout(timer);
		};
	});
</script>

<svelte:window
	onkeydown={(event) => {
		if (event.key === 'Escape' && addGameDialogOpen) {
			closeAddGameDialog();
		}
	}}
/>

<div class={`app-page pb-12 ${restoringViewPosition ? 'opacity-0' : 'opacity-100'}`}>
	<div class="app-page-inner w-full">
		<header class="page-header shrink-0">
			<div>
				<h1 class="page-title">{t('游戏库', 'Games Library')}</h1>
			</div>

			<div class="toolbar">
				<input
					type="text"
					bind:value={search}
					placeholder={t('搜索游戏名称', 'Search game name')}
					class="control-input w-full px-3 md:w-64"
					onkeydown={(event) => {
						if (event.key === 'Enter') {
							loadGames(false);
						}
					}}
				/>

				<label class="control-input flex items-center gap-2 px-3 text-sm">
					<input type="checkbox" bind:checked={installedOnly} class="h-4 w-4" />
					{t('仅已安装', 'Installed Only')}
				</label>

				<button class="btn btn-secondary gap-2" type="button" onclick={openAddGameDialog}>
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
						<path d="M12 5v14" />
						<path d="M5 12h14" />
					</svg>
					{t('添加游戏', 'Add Game')}
				</button>

				<button
					class="btn btn-primary relative w-24 disabled:cursor-not-allowed disabled:opacity-60"
					disabled={loading || refreshing}
					onclick={() => loadGames(false)}
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

		{#if errorMessage}
			<p
				class="status-message border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300"
			>
				{t('加载失败：', 'Load failed: ')}{errorMessage}
			</p>
		{/if}

		<div class="space-y-6">
			{#if loading}
				<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
					{#each [1, 2, 3, 4, 5, 6] as row (row)}
						<article class="surface-card p-5">
							<div class="skeleton-shimmer h-5 w-2/3 rounded-md"></div>
							<div class="skeleton-shimmer mt-3 h-4 w-1/3 rounded-md"></div>
							<div class="mt-6 grid grid-cols-2 gap-3">
								<div class="skeleton-shimmer h-12 rounded-xl"></div>
								<div class="skeleton-shimmer h-12 rounded-xl"></div>
							</div>
						</article>
					{/each}
				</div>
			{:else if getVisibleGames().length === 0}
				<article class="empty-state">
					<p class="text-sm text-gray-500 dark:text-gray-400">
						{t('没有找到符合条件的游戏', 'No matching games found')}
					</p>
				</article>
			{:else}
				<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
					{#each getVisibleGames() as game (game.id)}
						{@const coverSrc = getAssetSrc(game.cover_path)}
						<a
							href={resolve(`/games/${game.id}`)}
							class="surface-card block overflow-hidden transition hover:-translate-y-0.5 hover:border-blue-300 hover:shadow-md dark:hover:border-blue-700"
							onclick={saveGamesViewState}
						>
							<div class="grid grid-cols-[88px_minmax(0,1fr)] gap-4 p-5">
								<div
									class="subtle-panel flex aspect-[2/3] w-[88px] items-center justify-center overflow-hidden"
								>
									{#if coverSrc}
										<img src={coverSrc} alt="" class="h-full w-full object-cover" loading="lazy" />
									{:else}
										<span class="text-2xl font-bold text-gray-400 dark:text-gray-500">
											{game.name.slice(0, 1).toUpperCase()}
										</span>
									{/if}
								</div>

								<div class="min-w-0">
									<div class="flex items-start justify-between gap-3">
										<h3 class="line-clamp-2 text-lg font-semibold text-gray-900 dark:text-gray-100">
											{game.name}
										</h3>
										<span
											class={`shrink-0 rounded-full px-2.5 py-1 text-xs ${
												game.is_installed
													? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
													: 'bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300'
											}`}
										>
											{game.is_installed ? t('已安装', 'Installed') : t('未安装', 'Not Installed')}
										</span>
									</div>

									<p class="mt-2 text-xs text-gray-500 dark:text-gray-400">
										AppID: {game.appid ?? '-'}
									</p>

									<div class="mt-5 grid grid-cols-2 gap-3">
										<div class="subtle-panel p-3">
											<p class="data-label">{t('累计时长', 'Total Playtime')}</p>
											<p class="data-value mt-1">{game.total_playtime_formatted}</p>
										</div>
										<div class="subtle-panel p-3">
											<p class="data-label">{t('会话数', 'Sessions')}</p>
											<p class="data-value mt-1">{game.session_count}</p>
										</div>
									</div>

									<p class="mt-4 text-xs text-gray-500 dark:text-gray-400">
										{t('最后游玩：', 'Last Played: ')}{formatDateTime(game.last_played_at)}
									</p>
								</div>
							</div>
						</a>
					{/each}
				</div>
			{/if}
			<div class="h-8" aria-hidden="true"></div>
		</div>
	</div>
</div>

{#if addGameDialogOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4 backdrop-blur-sm"
		role="presentation"
	>
		<form
			class="surface-card w-full max-w-2xl p-5 shadow-2xl"
			onsubmit={(event) => {
				event.preventDefault();
				void submitManualGame();
			}}
		>
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
						{t('添加游戏', 'Add Game')}
					</h2>
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						{t(
							'选择 exe 后会自动生成名称，并尝试提取程序图标',
							'Choose an exe to auto-fill the name and extract its icon'
						)}
					</p>
				</div>
				<button
					type="button"
					class="btn btn-icon text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
					onclick={closeAddGameDialog}
					aria-label={t('关闭', 'Close')}
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
						<path d="M18 6 6 18" />
						<path d="m6 6 12 12" />
					</svg>
				</button>
			</div>

			{#if addGameError}
				<p
					class="status-message mt-4 border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300"
				>
					{addGameError}
				</p>
			{/if}

			<div class="mt-5 space-y-4">
				<label class="block space-y-2">
					<span class="text-sm font-medium text-gray-700 dark:text-gray-200">
						{t('游戏名称', 'Game Name')}
					</span>
					<input
						type="text"
						bind:value={manualGameForm.name}
						class="control-input w-full px-3"
						placeholder={t('游戏名称', 'Game name')}
						disabled={addGameSaving}
					/>
				</label>

				<div class="space-y-2">
					<span class="text-sm font-medium text-gray-700 dark:text-gray-200">
						{t('启动程序', 'Executable')}
					</span>
					<div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
						<input
							type="text"
							bind:value={manualGameForm.executable_path}
							class="control-input w-full px-3"
							placeholder={t('选择游戏 exe 文件', 'Choose game exe file')}
							disabled={addGameSaving}
						/>
						<button
							type="button"
							class="btn btn-secondary"
							onclick={chooseExecutable}
							disabled={addGameSaving}
						>
							{t('浏览', 'Browse')}
						</button>
					</div>
				</div>

				<div class="grid gap-3 md:grid-cols-3">
					<div class="space-y-2">
						<span class="text-sm font-medium text-gray-700 dark:text-gray-200">
							{t('图标', 'Icon')}
						</span>
						<button
							type="button"
							class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
							onclick={() => chooseImage('icon_path')}
							disabled={addGameSaving}
						>
							<span class="min-w-0 truncate">
								{manualGameForm.icon_path
									? manualGameForm.icon_path.split(/[\\/]/).pop()
									: t('自动识别或自选', 'Auto or choose')}
							</span>
						</button>
					</div>

					<div class="space-y-2">
						<span class="text-sm font-medium text-gray-700 dark:text-gray-200">
							{t('封面图', 'Cover')}
						</span>
						<button
							type="button"
							class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
							onclick={() => chooseImage('cover_path')}
							disabled={addGameSaving}
						>
							<span class="min-w-0 truncate">
								{manualGameForm.cover_path
									? manualGameForm.cover_path.split(/[\\/]/).pop()
									: t('可选', 'Optional')}
							</span>
						</button>
					</div>

					<div class="space-y-2">
						<span class="text-sm font-medium text-gray-700 dark:text-gray-200">
							{t('背景图', 'Hero')}
						</span>
						<button
							type="button"
							class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
							onclick={() => chooseImage('hero_path')}
							disabled={addGameSaving}
						>
							<span class="min-w-0 truncate">
								{manualGameForm.hero_path
									? manualGameForm.hero_path.split(/[\\/]/).pop()
									: t('可选', 'Optional')}
							</span>
						</button>
					</div>
				</div>
			</div>

			<div class="mt-6 flex justify-end gap-3">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={closeAddGameDialog}
					disabled={addGameSaving}
				>
					{t('取消', 'Cancel')}
				</button>
				<button
					type="submit"
					class="btn btn-primary disabled:cursor-not-allowed disabled:opacity-60"
					disabled={addGameSaving}
				>
					{addGameSaving ? t('添加中...', 'Adding...') : t('添加游戏', 'Add Game')}
				</button>
			</div>
		</form>
	</div>
{/if}
