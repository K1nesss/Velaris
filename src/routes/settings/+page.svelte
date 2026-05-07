<script lang="ts">
	import { onMount } from 'svelte';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { fly } from 'svelte/transition';
	import {
		exportDatabase,
		getHiddenGamesList,
		getIgnoredGamesList,
		importDatabase,
		restoreHiddenGame,
		restoreIgnoredGame,
		type HiddenGameItem,
		type IgnoredGameItem
	} from '$lib/api';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import {
		checkForAppUpdate,
		formatUpdateNotes,
		type AppUpdate,
		type AppUpdateDownloadEvent
	} from '$lib/updater';
	import {
		type AppSettings,
		type AppLanguage,
		type ThemeMode,
		DEFAULT_SETTINGS,
		mergeSettings,
		loadSettingsFromStorage,
		saveSettingsToStorage
	} from '$lib/settings';

	let settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });
	let settingsLoaded = false;
	let maintenanceBusy = $state(false);
	let maintenanceResult = $state('');
	let importInput = $state<HTMLInputElement | null>(null);
	let ignoredGames = $state<IgnoredGameItem[]>([]);
	let ignoredGamesLoading = $state(false);
	let ignoredGamesMessage = $state('');
	let restoringAppId = $state<number | null>(null);
	let hiddenGames = $state<HiddenGameItem[]>([]);
	let hiddenGamesLoading = $state(false);
	let hiddenGamesMessage = $state('');
	let restoringHiddenGameId = $state<number | null>(null);
	let checkingUpdate = $state(false);
	let installingUpdate = $state(false);
	let updateMessage = $state('');
	let updateProgress = $state(0);
	let availableUpdate = $state<AppUpdate | null>(null);
	let updateDialog = $state({
		open: false,
		currentVersion: '',
		version: '',
		date: '',
		notes: ''
	});
	let confirmDialog = $state({
		open: false,
		title: '',
		message: '',
		confirmText: '',
		cancelText: '',
		variant: 'default' as 'default' | 'danger'
	});
	let confirmResolver: ((value: boolean) => void) | null = null;
	type SettingsSelectKey = 'dashboardRefreshSeconds' | 'timelinePageSize' | 'analyticsDefaultRange';
	type NumberOption = {
		value: number;
		zh: string;
		en: string;
	};
	let openSettingsSelect = $state<SettingsSelectKey | null>(null);

	const dashboardRefreshOptions: NumberOption[] = [
		{ value: 2, zh: '2 秒', en: '2 sec' },
		{ value: 5, zh: '5 秒', en: '5 sec' },
		{ value: 10, zh: '10 秒', en: '10 sec' },
		{ value: 30, zh: '30 秒', en: '30 sec' }
	];
	const timelinePageSizeOptions: NumberOption[] = [
		{ value: 50, zh: '50 条', en: '50 items' },
		{ value: 80, zh: '80 条', en: '80 items' },
		{ value: 120, zh: '120 条', en: '120 items' },
		{ value: 200, zh: '200 条', en: '200 items' }
	];
	const analyticsDefaultRangeOptions: NumberOption[] = [
		{ value: 1, zh: '1 天', en: '1 day' },
		{ value: 7, zh: '7 天', en: '7 days' },
		{ value: 30, zh: '30 天', en: '30 days' },
		{ value: 365, zh: '1 年', en: '1 year' }
	];

	function serializeSettings(value: AppSettings) {
		return JSON.stringify(value);
	}

	function t(zh: string, en: string) {
		return settings.language === 'zh-CN' ? zh : en;
	}

	function isDarkByTheme(themeMode: ThemeMode) {
		if (themeMode === 'dark') {
			return true;
		}
		if (themeMode === 'light') {
			return false;
		}
		return window.matchMedia('(prefers-color-scheme: dark)').matches;
	}

	function applyThemePreference(themeMode: ThemeMode) {
		const shouldUseDark = isDarkByTheme(themeMode);

		document.documentElement.classList.toggle('dark', shouldUseDark);
		localStorage.setItem('theme', shouldUseDark ? 'dark' : 'light');
		window.dispatchEvent(
			new CustomEvent('pt-theme-change', {
				detail: { darkMode: shouldUseDark }
			})
		);
	}

	function applyLanguagePreference(language: AppLanguage) {
		document.documentElement.lang = language;
		window.dispatchEvent(
			new CustomEvent('pt-language-change', {
				detail: { language }
			})
		);
	}

	function applyAnimationPreference(enabled: boolean) {
		document.documentElement.classList.toggle('animations-disabled', !enabled);
		window.dispatchEvent(
			new CustomEvent('pt-animations-change', {
				detail: { animationsEnabled: enabled }
			})
		);
	}

	function loadSettings() {
		settings = loadSettingsFromStorage();
		applyThemePreference(settings.themeMode);
		applyLanguagePreference(settings.language);
		applyAnimationPreference(settings.animationsEnabled);
		settingsLoaded = true;
	}

	function persistSettings() {
		if (!settingsLoaded) {
			return;
		}

		saveSettingsToStorage(mergeSettings(settings));
	}

	function resetToDefault() {
		settings = { ...DEFAULT_SETTINGS };
		applyThemePreference(settings.themeMode);
		applyLanguagePreference(settings.language);
		applyAnimationPreference(settings.animationsEnabled);
		persistSettings();
	}

	function base64ToBlob(base64: string, type = 'application/octet-stream') {
		const binary = atob(base64);
		const bytes = new Uint8Array(binary.length);
		for (let index = 0; index < binary.length; index += 1) {
			bytes[index] = binary.charCodeAt(index);
		}
		return new Blob([bytes], { type });
	}

	function arrayBufferToBase64(buffer: ArrayBuffer) {
		const bytes = new Uint8Array(buffer);
		let binary = '';
		for (let index = 0; index < bytes.length; index += 1) {
			binary += String.fromCharCode(bytes[index]);
		}
		return btoa(binary);
	}

	function formatDateTime(timestamp: number) {
		return new Intl.DateTimeFormat(settings.language, {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit'
		}).format(new Date(timestamp * 1000));
	}

	function optionLabel(options: NumberOption[], value: number) {
		const option = options.find((item) => item.value === value) ?? options[0];
		return t(option.zh, option.en);
	}

	function toggleSettingsSelect(key: SettingsSelectKey) {
		openSettingsSelect = openSettingsSelect === key ? null : key;
	}

	function setNumberSetting(key: SettingsSelectKey, value: number) {
		if (key === 'dashboardRefreshSeconds') {
			settings.dashboardRefreshSeconds = value;
		} else if (key === 'timelinePageSize') {
			settings.timelinePageSize = value;
		} else {
			settings.analyticsDefaultRange = value;
		}
		openSettingsSelect = null;
	}

	function toggleBooleanSetting(
		key: 'animationsEnabled' | 'launchOnStartup' | 'minimizeToTray' | 'autoStartMonitor'
	) {
		settings[key] = !settings[key];
		if (key === 'animationsEnabled') {
			applyAnimationPreference(settings.animationsEnabled);
		}
	}

	async function loadIgnoredGames() {
		ignoredGamesLoading = true;
		ignoredGamesMessage = '';

		try {
			ignoredGames = await getIgnoredGamesList();
		} catch (error) {
			ignoredGamesMessage = error instanceof Error ? error.message : String(error);
		} finally {
			ignoredGamesLoading = false;
		}
	}

	async function loadHiddenGames() {
		hiddenGamesLoading = true;
		hiddenGamesMessage = '';

		try {
			hiddenGames = await getHiddenGamesList();
		} catch (error) {
			hiddenGamesMessage = error instanceof Error ? error.message : String(error);
		} finally {
			hiddenGamesLoading = false;
		}
	}

	function askConfirm(options: {
		title: string;
		message: string;
		confirmText: string;
		cancelText: string;
		variant?: 'default' | 'danger';
	}) {
		confirmDialog = {
			open: true,
			title: options.title,
			message: options.message,
			confirmText: options.confirmText,
			cancelText: options.cancelText,
			variant: options.variant ?? 'default'
		};

		return new Promise<boolean>((resolve) => {
			confirmResolver = resolve;
		});
	}

	function closeConfirm(result: boolean) {
		confirmDialog.open = false;
		confirmResolver?.(result);
		confirmResolver = null;
	}

	async function restoreGame(appid: number, name: string) {
		const confirmed = await askConfirm({
			title: t('恢复忽略游戏', 'Restore Ignored Game'),
			message: t(
				`确定要恢复“${name}”吗？恢复后会立即重新扫描 Steam 库。`,
				`Restore "${name}" and scan the Steam library now?`
			),
			confirmText: t('恢复', 'Restore'),
			cancelText: t('取消', 'Cancel')
		});
		if (!confirmed) {
			return;
		}

		restoringAppId = appid;
		ignoredGamesMessage = '';

		try {
			await restoreIgnoredGame(appid);
			await loadIgnoredGames();
			ignoredGamesMessage = t('已恢复，并完成 Steam 扫描', 'Restored and Steam scan completed');
		} catch (error) {
			ignoredGamesMessage = error instanceof Error ? error.message : String(error);
		} finally {
			restoringAppId = null;
		}
	}

	async function restoreHidden(gameId: number, name: string) {
		const confirmed = await askConfirm({
			title: t('恢复显示', 'Show Game'),
			message: t(`确定要恢复显示“${name}”吗？`, `Show "${name}" in the game library again?`),
			confirmText: t('恢复显示', 'Show'),
			cancelText: t('取消', 'Cancel')
		});
		if (!confirmed) {
			return;
		}

		restoringHiddenGameId = gameId;
		hiddenGamesMessage = '';

		try {
			await restoreHiddenGame(gameId);
			await loadHiddenGames();
			hiddenGamesMessage = t('已恢复显示', 'Game is visible again');
		} catch (error) {
			hiddenGamesMessage = error instanceof Error ? error.message : String(error);
		} finally {
			restoringHiddenGameId = null;
		}
	}

	async function checkUpdatesFromSettings() {
		if (checkingUpdate || installingUpdate) {
			return;
		}

		checkingUpdate = true;
		updateMessage = '';
		updateProgress = 0;
		try {
			const update = await checkForAppUpdate();
			if (!update) {
				availableUpdate = null;
				updateMessage = t('当前已是最新版本', 'You are already on the latest version');
				return;
			}

			availableUpdate = update;
			updateDialog = {
				open: true,
				currentVersion: update.currentVersion,
				version: update.version,
				date: update.date ?? '',
				notes: formatUpdateNotes(update.body)
			};
		} catch (error) {
			updateMessage = error instanceof Error ? error.message : String(error);
		} finally {
			checkingUpdate = false;
		}
	}

	function closeUpdateDialog() {
		if (installingUpdate) {
			return;
		}
		updateDialog.open = false;
		updateProgress = 0;
	}

	function handleUpdateDownloadEvent(event: AppUpdateDownloadEvent) {
		if (event.event === 'Started') {
			updateProgress = 0;
		} else if (event.event === 'Progress') {
			updateProgress += event.data.chunkLength;
		}
	}

	async function installAvailableUpdate() {
		if (!availableUpdate || installingUpdate) {
			return;
		}

		installingUpdate = true;
		updateMessage = '';
		updateProgress = 0;
		try {
			await availableUpdate.downloadAndInstall(handleUpdateDownloadEvent);
			await relaunch();
		} catch (error) {
			updateMessage = error instanceof Error ? error.message : String(error);
			installingUpdate = false;
		}
	}

	async function runExportDatabase() {
		maintenanceBusy = true;
		maintenanceResult = '';

		try {
			const payload = await exportDatabase();
			const blob = base64ToBlob(payload, 'application/octet-stream');
			const url = URL.createObjectURL(blob);
			const link = document.createElement('a');
			const timestamp = new Date().toISOString().replace(/[.:]/g, '-');
			link.href = url;
			link.download = `playtime-tracker-${timestamp}.db`;
			link.click();
			URL.revokeObjectURL(url);
			maintenanceResult = t('数据库已导出', 'Database exported');
		} catch (error) {
			maintenanceResult = error instanceof Error ? error.message : String(error);
		} finally {
			maintenanceBusy = false;
		}
	}

	async function runImportDatabase(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) {
			return;
		}

		maintenanceBusy = true;
		maintenanceResult = '';

		try {
			const buffer = await file.arrayBuffer();
			const payload = arrayBufferToBase64(buffer);
			await importDatabase(payload);
			maintenanceResult = t(
				'数据库导入成功，请重启应用以确保监控服务使用新数据',
				'Database imported successfully. Please restart the app to ensure monitor service uses new data.'
			);
		} catch (error) {
			maintenanceResult = error instanceof Error ? error.message : String(error);
		} finally {
			maintenanceBusy = false;
			target.value = '';
		}
	}

	onMount(() => {
		loadSettings();
		loadIgnoredGames();
		loadHiddenGames();

		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		const handleSystemThemeChange = () => {
			if (settings.themeMode === 'system') {
				applyThemePreference('system');
			}
		};

		mediaQuery.addEventListener('change', handleSystemThemeChange);

		return () => {
			mediaQuery.removeEventListener('change', handleSystemThemeChange);
		};
	});

	$effect(() => {
		serializeSettings(settings);
		applyThemePreference(settings.themeMode);
		applyLanguagePreference(settings.language);
		applyAnimationPreference(settings.animationsEnabled);
		persistSettings();
	});
</script>

<svelte:window onclick={() => (openSettingsSelect = null)} />

<section class="app-page">
	<div class="app-page-inner space-y-6">
		<header class="page-header">
			<div>
				<h1 class="page-title">{t('设置', 'Settings')}</h1>
			</div>

			<div class="toolbar">
				<button
					class="btn btn-secondary disabled:cursor-not-allowed disabled:opacity-60"
					onclick={resetToDefault}
				>
					{t('恢复默认', 'Reset Defaults')}
				</button>
			</div>
		</header>

		<div class="grid gap-6 xl:grid-cols-2">
			<article class="surface-card p-5 xl:col-span-2">
				<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
					{t('Steam API', 'Steam API')}
				</h2>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{t(
						'用于后续通过 Steam API 获取游戏图标、背景图、成就等数据',
						'Used later to fetch game icons, background images, achievements and other data from Steam API'
					)}
				</p>

				<div class="mt-4 grid gap-4 md:grid-cols-2">
					<input
						type="password"
						bind:value={settings.apiKey}
						placeholder={t('输入 Steam Web API Key', 'Enter Steam Web API key')}
						aria-label="Steam Web API Key"
						autocomplete="off"
						spellcheck="false"
						class="control-input w-full px-3"
					/>

					<input
						type="text"
						bind:value={settings.steam64Id}
						placeholder={t('输入 17 位 Steam ID', 'Enter 17-digit Steam ID')}
						aria-label="Steam ID"
						inputmode="numeric"
						autocomplete="off"
						spellcheck="false"
						class="control-input w-full px-3"
					/>
				</div>
			</article>

			<article class="surface-card p-5 xl:col-span-2">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
							{t('已忽略游戏', 'Ignored Games')}
						</h2>
						<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
							{t(
								'删除游戏后会加入这里，Steam 扫描会跳过这些 AppID',
								'Deleted games are listed here, and Steam scans will skip these AppIDs'
							)}
						</p>
					</div>
					<button
						type="button"
						class="btn btn-secondary disabled:cursor-not-allowed disabled:opacity-60"
						onclick={loadIgnoredGames}
						disabled={ignoredGamesLoading || restoringAppId !== null}
					>
						{ignoredGamesLoading ? t('刷新中...', 'Refreshing...') : t('刷新', 'Refresh')}
					</button>
				</div>

				{#if ignoredGamesMessage}
					<p
						class="mt-4 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-200"
					>
						{ignoredGamesMessage}
					</p>
				{/if}

				{#if ignoredGamesLoading}
					<div class="mt-4 space-y-2">
						<div class="skeleton-shimmer h-12 rounded-lg"></div>
						<div class="skeleton-shimmer h-12 rounded-lg"></div>
					</div>
				{:else if ignoredGames.length === 0}
					<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">
						{t('暂无已忽略游戏', 'No ignored games')}
					</p>
				{:else}
					<div class="mt-4 overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700">
						<table class="min-w-full divide-y divide-gray-200 text-sm dark:divide-gray-700">
							<thead class="bg-gray-50 dark:bg-[#101522]">
								<tr>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										AppID
									</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										{t('名称', 'Name')}
									</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										{t('忽略时间', 'Ignored At')}
									</th>
									<th class="px-4 py-3 text-right font-medium text-gray-600 dark:text-gray-300">
										{t('操作', 'Actions')}
									</th>
								</tr>
							</thead>
							<tbody
								class="divide-y divide-gray-100 bg-white dark:divide-gray-800 dark:bg-[#151926]"
							>
								{#each ignoredGames as game (game.appid)}
									<tr>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">{game.appid}</td>
										<td class="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">
											{game.name}
										</td>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">
											{formatDateTime(game.ignored_at)}
										</td>
										<td class="px-4 py-3 text-right">
											<button
												type="button"
												class="rounded-md border border-blue-200 px-3 py-1.5 text-xs font-medium text-blue-700 hover:bg-blue-50 disabled:opacity-60 dark:border-blue-900 dark:text-blue-300 dark:hover:bg-blue-950/30"
												onclick={() => restoreGame(game.appid, game.name)}
												disabled={restoringAppId !== null}
											>
												{restoringAppId === game.appid
													? t('恢复中...', 'Restoring...')
													: t('恢复', 'Restore')}
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</article>

			<article class="surface-card p-5 xl:col-span-2">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
							{t('已隐藏游戏', 'Hidden Games')}
						</h2>
						<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
							{t(
								'隐藏游戏会保留数据，只是不在游戏库中显示',
								'Hidden games keep their data, but are not shown in the game library'
							)}
						</p>
					</div>
					<button
						type="button"
						class="btn btn-secondary disabled:cursor-not-allowed disabled:opacity-60"
						onclick={loadHiddenGames}
						disabled={hiddenGamesLoading || restoringHiddenGameId !== null}
					>
						{hiddenGamesLoading ? t('刷新中...', 'Refreshing...') : t('刷新', 'Refresh')}
					</button>
				</div>

				{#if hiddenGamesMessage}
					<p
						class="mt-4 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-200"
					>
						{hiddenGamesMessage}
					</p>
				{/if}

				{#if hiddenGamesLoading}
					<div class="mt-4 space-y-2">
						<div class="skeleton-shimmer h-12 rounded-lg"></div>
						<div class="skeleton-shimmer h-12 rounded-lg"></div>
					</div>
				{:else if hiddenGames.length === 0}
					<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">
						{t('暂无已隐藏游戏', 'No hidden games')}
					</p>
				{:else}
					<div class="mt-4 overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700">
						<table class="min-w-full divide-y divide-gray-200 text-sm dark:divide-gray-700">
							<thead class="bg-gray-50 dark:bg-[#101522]">
								<tr>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										AppID
									</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										{t('名称', 'Name')}
									</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
										{t('隐藏时间', 'Hidden At')}
									</th>
									<th class="px-4 py-3 text-right font-medium text-gray-600 dark:text-gray-300">
										{t('操作', 'Actions')}
									</th>
								</tr>
							</thead>
							<tbody
								class="divide-y divide-gray-100 bg-white dark:divide-gray-800 dark:bg-[#151926]"
							>
								{#each hiddenGames as game (game.id)}
									<tr>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">
											{game.appid ?? '-'}
										</td>
										<td class="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">
											{game.name}
										</td>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">
											{formatDateTime(game.updated_at)}
										</td>
										<td class="px-4 py-3 text-right">
											<button
												type="button"
												class="rounded-md border border-blue-200 px-3 py-1.5 text-xs font-medium text-blue-700 hover:bg-blue-50 disabled:opacity-60 dark:border-blue-900 dark:text-blue-300 dark:hover:bg-blue-950/30"
												onclick={() => restoreHidden(game.id, game.name)}
												disabled={restoringHiddenGameId !== null}
											>
												{restoringHiddenGameId === game.id
													? t('恢复中...', 'Restoring...')
													: t('恢复显示', 'Show')}
											</button>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</article>

			<article class="surface-card p-5">
				<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
					{t('外观', 'Appearance')}
				</h2>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{t('主题、语言与动效显示偏好', 'Theme, language and animation preferences')}
				</p>

				<div class="mt-4 space-y-4">
					<div>
						<p class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
							{t('语言', 'Language')}
						</p>
						<div class="inline-flex rounded-lg border border-gray-200 p-1 dark:border-gray-700">
							<button
								class={`rounded-md px-3 py-1.5 text-sm transition ${settings.language === 'zh-CN' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
								onclick={() => {
									settings.language = 'zh-CN';
								}}
							>
								中文
							</button>
							<button
								class={`rounded-md px-3 py-1.5 text-sm transition ${settings.language === 'en-US' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
								onclick={() => {
									settings.language = 'en-US';
								}}
							>
								English
							</button>
						</div>
					</div>

					<div>
						<p class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
							{t('主题模式', 'Theme Mode')}
						</p>
						<div class="inline-flex rounded-lg border border-gray-200 p-1 dark:border-gray-700">
							<button
								class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'light' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
								onclick={() => {
									settings.themeMode = 'light';
								}}
							>
								{t('浅色', 'Light')}
							</button>
							<button
								class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'dark' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
								onclick={() => {
									settings.themeMode = 'dark';
								}}
							>
								{t('深色', 'Dark')}
							</button>
							<button
								class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'system' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
								onclick={() => {
									settings.themeMode = 'system';
								}}
							>
								{t('跟随系统', 'System')}
							</button>
						</div>
					</div>

					<button
						type="button"
						role="switch"
						aria-checked={settings.animationsEnabled}
						class="flex w-full items-center justify-between rounded-lg border border-gray-200 px-3 py-2 text-left dark:border-gray-700"
						onclick={() => toggleBooleanSetting('animationsEnabled')}
					>
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('启用界面动效', 'Enable UI Animations')}</span
						>
						<span
							class={`relative h-7 w-12 shrink-0 rounded-full border transition-colors ${
								settings.animationsEnabled
									? 'border-blue-500 bg-blue-600'
									: 'border-gray-300 bg-gray-200 dark:border-gray-700 dark:bg-gray-800'
							}`}
							aria-hidden="true"
						>
							<span
								class={`absolute top-1/2 left-1 h-5 w-5 -translate-y-1/2 rounded-full bg-white shadow-sm transition-transform ${
									settings.animationsEnabled ? 'translate-x-5' : 'translate-x-0'
								}`}
							></span>
						</span>
					</button>
				</div>
			</article>

			<article class="surface-card p-5">
				<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
					{t('应用行为', 'App Behavior')}
				</h2>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{t('启动与后台运行策略', 'Startup and background-running strategy')}
				</p>

				<div class="mt-4 space-y-3">
					<button
						type="button"
						role="switch"
						aria-checked={settings.launchOnStartup}
						class="flex w-full items-center justify-between rounded-lg border border-gray-200 px-3 py-2 text-left dark:border-gray-700"
						onclick={() => toggleBooleanSetting('launchOnStartup')}
					>
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('开机自动启动', 'Launch on Startup')}</span
						>
						<span
							class={`relative h-7 w-12 shrink-0 rounded-full border transition-colors ${
								settings.launchOnStartup
									? 'border-blue-500 bg-blue-600'
									: 'border-gray-300 bg-gray-200 dark:border-gray-700 dark:bg-gray-800'
							}`}
							aria-hidden="true"
						>
							<span
								class={`absolute top-1/2 left-1 h-5 w-5 -translate-y-1/2 rounded-full bg-white shadow-sm transition-transform ${
									settings.launchOnStartup ? 'translate-x-5' : 'translate-x-0'
								}`}
							></span>
						</span>
					</button>

					<button
						type="button"
						role="switch"
						aria-checked={settings.minimizeToTray}
						class="flex w-full items-center justify-between rounded-lg border border-gray-200 px-3 py-2 text-left dark:border-gray-700"
						onclick={() => toggleBooleanSetting('minimizeToTray')}
					>
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('关闭时最小化到托盘', 'Minimize to Tray on Close')}</span
						>
						<span
							class={`relative h-7 w-12 shrink-0 rounded-full border transition-colors ${
								settings.minimizeToTray
									? 'border-blue-500 bg-blue-600'
									: 'border-gray-300 bg-gray-200 dark:border-gray-700 dark:bg-gray-800'
							}`}
							aria-hidden="true"
						>
							<span
								class={`absolute top-1/2 left-1 h-5 w-5 -translate-y-1/2 rounded-full bg-white shadow-sm transition-transform ${
									settings.minimizeToTray ? 'translate-x-5' : 'translate-x-0'
								}`}
							></span>
						</span>
					</button>

					<button
						type="button"
						role="switch"
						aria-checked={settings.autoStartMonitor}
						class="flex w-full items-center justify-between rounded-lg border border-gray-200 px-3 py-2 text-left dark:border-gray-700"
						onclick={() => toggleBooleanSetting('autoStartMonitor')}
					>
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('应用启动后自动监控进程', 'Auto Start Process Monitor')}</span
						>
						<span
							class={`relative h-7 w-12 shrink-0 rounded-full border transition-colors ${
								settings.autoStartMonitor
									? 'border-blue-500 bg-blue-600'
									: 'border-gray-300 bg-gray-200 dark:border-gray-700 dark:bg-gray-800'
							}`}
							aria-hidden="true"
						>
							<span
								class={`absolute top-1/2 left-1 h-5 w-5 -translate-y-1/2 rounded-full bg-white shadow-sm transition-transform ${
									settings.autoStartMonitor ? 'translate-x-5' : 'translate-x-0'
								}`}
							></span>
						</span>
					</button>
				</div>
			</article>

			<article class="surface-card p-5 xl:col-span-2">
				<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
					{t('数据展示', 'Data Display')}
				</h2>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{t(
						'控制 dashboard、timeline、analytics 默认展示参数',
						'Control default display parameters for dashboard, timeline and analytics'
					)}
				</p>

				<div class="mt-4 grid gap-4 md:grid-cols-3">
					<div class="space-y-2">
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('Dashboard 刷新间隔', 'Dashboard Refresh Interval')}</span
						>
						<div class="relative">
							<button
								type="button"
								class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
								aria-haspopup="listbox"
								aria-expanded={openSettingsSelect === 'dashboardRefreshSeconds'}
								onclick={(event) => {
									event.stopPropagation();
									toggleSettingsSelect('dashboardRefreshSeconds');
								}}
							>
								<span class="min-w-0 truncate"
									>{optionLabel(dashboardRefreshOptions, settings.dashboardRefreshSeconds)}</span
								>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
									class={`h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200 ${
										openSettingsSelect === 'dashboardRefreshSeconds' ? 'rotate-180' : ''
									}`}
									aria-hidden="true"
								>
									<path d="m6 9 6 6 6-6" />
								</svg>
							</button>

							{#if openSettingsSelect === 'dashboardRefreshSeconds'}
								<div
									class="surface-card absolute right-0 left-0 z-30 mt-2 p-1"
									transition:fly={{ y: -6, duration: 160 }}
									role="listbox"
								>
									{#each dashboardRefreshOptions as option (option.value)}
										<button
											type="button"
											class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
												settings.dashboardRefreshSeconds === option.value
													? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
													: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
											}`}
											role="option"
											aria-selected={settings.dashboardRefreshSeconds === option.value}
											onclick={(event) => {
												event.stopPropagation();
												setNumberSetting('dashboardRefreshSeconds', option.value);
											}}
										>
											<span class="truncate">{t(option.zh, option.en)}</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>

					<div class="space-y-2">
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('Timeline 每次加载条数', 'Timeline Page Size')}</span
						>
						<div class="relative">
							<button
								type="button"
								class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
								aria-haspopup="listbox"
								aria-expanded={openSettingsSelect === 'timelinePageSize'}
								onclick={(event) => {
									event.stopPropagation();
									toggleSettingsSelect('timelinePageSize');
								}}
							>
								<span class="min-w-0 truncate"
									>{optionLabel(timelinePageSizeOptions, settings.timelinePageSize)}</span
								>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
									class={`h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200 ${
										openSettingsSelect === 'timelinePageSize' ? 'rotate-180' : ''
									}`}
									aria-hidden="true"
								>
									<path d="m6 9 6 6 6-6" />
								</svg>
							</button>

							{#if openSettingsSelect === 'timelinePageSize'}
								<div
									class="surface-card absolute right-0 left-0 z-30 mt-2 p-1"
									transition:fly={{ y: -6, duration: 160 }}
									role="listbox"
								>
									{#each timelinePageSizeOptions as option (option.value)}
										<button
											type="button"
											class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
												settings.timelinePageSize === option.value
													? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
													: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
											}`}
											role="option"
											aria-selected={settings.timelinePageSize === option.value}
											onclick={(event) => {
												event.stopPropagation();
												setNumberSetting('timelinePageSize', option.value);
											}}
										>
											<span class="truncate">{t(option.zh, option.en)}</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>

					<div class="space-y-2">
						<span class="text-sm text-gray-700 dark:text-gray-200"
							>{t('Analytics 默认区间', 'Analytics Default Range')}</span
						>
						<div class="relative">
							<button
								type="button"
								class="control-input flex w-full items-center justify-between gap-3 px-3 text-left"
								aria-haspopup="listbox"
								aria-expanded={openSettingsSelect === 'analyticsDefaultRange'}
								onclick={(event) => {
									event.stopPropagation();
									toggleSettingsSelect('analyticsDefaultRange');
								}}
							>
								<span class="min-w-0 truncate"
									>{optionLabel(analyticsDefaultRangeOptions, settings.analyticsDefaultRange)}</span
								>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
									class={`h-4 w-4 shrink-0 text-gray-400 transition-transform duration-200 ${
										openSettingsSelect === 'analyticsDefaultRange' ? 'rotate-180' : ''
									}`}
									aria-hidden="true"
								>
									<path d="m6 9 6 6 6-6" />
								</svg>
							</button>

							{#if openSettingsSelect === 'analyticsDefaultRange'}
								<div
									class="surface-card absolute right-0 left-0 z-30 mt-2 p-1"
									transition:fly={{ y: -6, duration: 160 }}
									role="listbox"
								>
									{#each analyticsDefaultRangeOptions as option (option.value)}
										<button
											type="button"
											class={`flex w-full items-center justify-between rounded-lg px-3 py-2 text-left text-sm transition-colors ${
												settings.analyticsDefaultRange === option.value
													? 'bg-blue-50 text-blue-700 dark:bg-blue-950/30 dark:text-blue-200'
													: 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-800'
											}`}
											role="option"
											aria-selected={settings.analyticsDefaultRange === option.value}
											onclick={(event) => {
												event.stopPropagation();
												setNumberSetting('analyticsDefaultRange', option.value);
											}}
										>
											<span class="truncate">{t(option.zh, option.en)}</span>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				</div>
			</article>

			<article class="surface-card p-5 xl:col-span-2">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
							{t('版本更新', 'Updates')}
						</h2>
						<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
							{t(
								'检查 Velaris 是否有新版本，更新前会显示版本号和更新内容',
								'Check whether a new Velaris version is available before installing it'
							)}
						</p>
					</div>
					<button
						type="button"
						class="btn btn-primary disabled:cursor-not-allowed disabled:opacity-60"
						onclick={checkUpdatesFromSettings}
						disabled={checkingUpdate || installingUpdate}
					>
						{checkingUpdate ? t('检查中...', 'Checking...') : t('检查更新', 'Check Updates')}
					</button>
				</div>

				{#if updateMessage}
					<p
						class="mt-4 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-200"
					>
						{updateMessage}
					</p>
				{/if}
			</article>

			<article class="surface-card p-5 xl:col-span-2">
				<h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
					{t('数据维护', 'Data Maintenance')}
				</h2>
				<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
					{t('导出或导入数据库文件', 'Export or import database file')}
				</p>

				<div class="mt-4 flex flex-wrap gap-3">
					<button
						class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200 dark:hover:bg-gray-800"
						onclick={runExportDatabase}
						disabled={maintenanceBusy}
					>
						{maintenanceBusy ? t('处理中...', 'Processing...') : t('导出数据库', 'Export Database')}
					</button>

					<button
						class="rounded-lg border border-blue-300 bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700 transition hover:bg-blue-100 dark:border-blue-900 dark:bg-blue-950/30 dark:text-blue-200 dark:hover:bg-blue-950/50"
						onclick={() => importInput?.click()}
						disabled={maintenanceBusy}
					>
						{t('导入数据库', 'Import Database')}
					</button>

					<input
						bind:this={importInput}
						type="file"
						class="hidden"
						accept=".db,.sqlite,.sqlite3"
						onchange={runImportDatabase}
					/>
				</div>

				{#if maintenanceResult}
					<p
						class="mt-3 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-200"
					>
						{maintenanceResult}
					</p>
				{/if}
			</article>
		</div>
	</div>
</section>

{#if updateDialog.open}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4 backdrop-blur-sm">
		<article class="surface-card w-full max-w-xl p-5 shadow-2xl">
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
						{t('发现新版本', 'Update Available')}
					</h2>
					<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
						{updateDialog.currentVersion} -> {updateDialog.version}
					</p>
				</div>
				<button
					type="button"
					class="btn btn-icon text-gray-500 hover:text-gray-900 disabled:opacity-60 dark:text-gray-400 dark:hover:text-gray-100"
					onclick={closeUpdateDialog}
					disabled={installingUpdate}
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

			<div
				class="mt-5 rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#101522]"
			>
				<p class="text-sm font-semibold text-gray-900 dark:text-gray-100">
					{t('更新内容', 'Release Notes')}
				</p>
				{#if updateDialog.notes}
					<pre
						class="mt-3 max-h-56 overflow-auto text-sm leading-6 whitespace-pre-wrap text-gray-700 dark:text-gray-200">{updateDialog.notes}</pre>
				{:else}
					<p class="mt-3 text-sm text-gray-500 dark:text-gray-400">
						{t('此版本没有填写更新说明', 'No release notes were provided for this version')}
					</p>
				{/if}
			</div>

			{#if installingUpdate}
				<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">
					{updateProgress > 0
						? t(
								`已下载 ${Math.round(updateProgress / 1024 / 1024)} MB`,
								`Downloaded ${Math.round(updateProgress / 1024 / 1024)} MB`
							)
						: t('正在准备下载...', 'Preparing download...')}
				</p>
			{/if}

			<div class="mt-6 flex justify-end gap-3">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={closeUpdateDialog}
					disabled={installingUpdate}
				>
					{t('取消', 'Cancel')}
				</button>
				<button
					type="button"
					class="btn btn-primary disabled:cursor-not-allowed disabled:opacity-60"
					onclick={installAvailableUpdate}
					disabled={installingUpdate}
				>
					{installingUpdate ? t('更新中...', 'Updating...') : t('立即更新', 'Update Now')}
				</button>
			</div>
		</article>
	</div>
{/if}

<ConfirmDialog
	open={confirmDialog.open}
	title={confirmDialog.title}
	message={confirmDialog.message}
	confirmText={confirmDialog.confirmText}
	cancelText={confirmDialog.cancelText}
	variant={confirmDialog.variant}
	onConfirm={() => closeConfirm(true)}
	onCancel={() => closeConfirm(false)}
/>
