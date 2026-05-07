<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import {
		deleteGame,
		deleteGameSession,
		getGameDetail,
		hideGame,
		syncSteamOwnedGameIcons,
		type GameDetail
	} from '$lib/api';
	import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';

	let { data } = $props<{ data: { gameId: number } }>();

	let loading = $state(true);
	let actionLoading = $state(false);
	let errorMessage = $state('');
	let detail = $state<GameDetail | null>(null);
	let language = $state<AppLanguage>('zh-CN');
	let confirmDialog = $state({
		open: false,
		title: '',
		message: '',
		confirmText: '',
		cancelText: '',
		variant: 'default' as 'default' | 'danger'
	});
	let noticeDialog = $state({
		open: false,
		title: '',
		message: '',
		confirmText: ''
	});
	let confirmResolver: ((value: boolean) => void) | null = null;

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

	function getMediaSrc(path: string | null) {
		return path ? convertFileSrc(path) : null;
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

	async function loadDetail() {
		loading = true;
		errorMessage = '';

		try {
			detail = await getGameDetail(data.gameId, 30);
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			loading = false;
		}
	}

	function returnToGames() {
		if (window.history.length > 1) {
			window.history.back();
			return;
		}

		goto(resolve('/games'));
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

	function showNotice(title: string, message: string, confirmText = t('知道了', 'OK')) {
		noticeDialog = {
			open: true,
			title,
			message,
			confirmText
		};
	}

	function closeNotice() {
		noticeDialog.open = false;
	}

	async function hideCurrentGame() {
		if (!detail || actionLoading) {
			return;
		}

		const confirmed = await askConfirm({
			title: t('隐藏游戏', 'Hide Game'),
			message: t(
				`确定要隐藏“${detail.name}”吗？它会从游戏库列表中移除。`,
				`Hide "${detail.name}" from the game library?`
			),
			confirmText: t('隐藏', 'Hide'),
			cancelText: t('取消', 'Cancel')
		});
		if (!confirmed) {
			return;
		}

		actionLoading = true;
		try {
			await hideGame(detail.id);
			returnToGames();
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			actionLoading = false;
		}
	}

	async function deleteCurrentGame() {
		if (!detail || actionLoading) {
			return;
		}

		const confirmed = await askConfirm({
			title: t('删除游戏', 'Delete Game'),
			message: t(
				`确定要删除“${detail.name}”吗？这个游戏和它的所有游玩记录都会被删除。`,
				`Delete "${detail.name}" and all of its play sessions?`
			),
			confirmText: t('删除', 'Delete'),
			cancelText: t('取消', 'Cancel'),
			variant: 'danger'
		});
		if (!confirmed) {
			return;
		}

		actionLoading = true;
		try {
			await deleteGame(detail.id);
			returnToGames();
		} catch (error) {
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			actionLoading = false;
		}
	}

	async function deleteSession(sessionId: number) {
		if (actionLoading) {
			return;
		}

		const confirmed = await askConfirm({
			title: t('删除会话', 'Delete Session'),
			message: t(`确定要删除会话 #${sessionId} 吗？`, `Delete session #${sessionId}?`),
			confirmText: t('删除', 'Delete'),
			cancelText: t('取消', 'Cancel'),
			variant: 'danger'
		});
		if (!confirmed) {
			return;
		}

		actionLoading = true;
		errorMessage = '';
		try {
			await deleteGameSession(sessionId);
			await loadDetail();
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			if (message.toLowerCase().includes('active session')) {
				showNotice(
					t('无法删除正在进行的记录', 'Cannot Delete Active Session'),
					t(
						'这个游戏当前仍在运行中。请先退出游戏，等待记录结束后再删除。',
						'This game is still running. Stop the game and wait for the session to finish before deleting it.'
					)
				);
			} else {
				errorMessage = message;
			}
		} finally {
			actionLoading = false;
		}
	}

	onMount(() => {
		const settings = loadSettingsFromStorage();
		language = settings.language;
		loadDetail();
		syncSteamMediaFromSettings().then((hasUpdates) => {
			if (hasUpdates) {
				loadDetail();
			}
		});
	});
</script>

<section class="app-page">
	<div class="app-page-inner space-y-6">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<button type="button" onclick={returnToGames} class="btn btn-secondary">
				{t('返回游戏库', 'Back to Games')}
			</button>
			<div class="flex flex-wrap items-center gap-2">
				<button type="button" onclick={loadDetail} class="btn btn-primary" disabled={actionLoading}>
					{t('刷新', 'Refresh')}
				</button>
				<button
					type="button"
					onclick={hideCurrentGame}
					class="btn btn-secondary"
					disabled={!detail || actionLoading}
				>
					{t('隐藏', 'Hide')}
				</button>
				<button
					type="button"
					onclick={deleteCurrentGame}
					class="btn border-red-200 bg-red-50 text-red-700 hover:bg-red-100 disabled:opacity-60 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300 dark:hover:bg-red-950/50"
					disabled={!detail || actionLoading}
				>
					{t('删除游戏', 'Delete Game')}
				</button>
			</div>
		</div>

		{#if loading}
			<article class="surface-card p-6">
				<div class="skeleton-shimmer h-48 rounded-xl"></div>
				<div class="skeleton-shimmer mt-6 h-8 w-64 rounded-md"></div>
				<div class="skeleton-shimmer mt-3 h-4 w-40 rounded-md"></div>
				<div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
					{#each [1, 2, 3, 4] as row (row)}
						<div class="skeleton-shimmer h-20 rounded-xl"></div>
					{/each}
				</div>
			</article>
		{:else if errorMessage && !detail}
			<article
				class="status-message border-red-200 bg-red-50 p-6 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300"
			>
				{t('加载失败：', 'Load failed: ')}{errorMessage}
			</article>
		{:else if detail}
			{@const heroSrc = getMediaSrc(detail.hero_path)}
			<div class="space-y-6">
				{#if errorMessage}
					<article
						class="status-message border-red-200 bg-red-50 p-4 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300"
					>
						{errorMessage}
					</article>
				{/if}

				<article class="surface-card overflow-hidden">
					{#if heroSrc}
						<div class="relative aspect-[2.5/1] min-h-48 overflow-hidden bg-gray-900">
							<img src={heroSrc} alt="" class="h-full w-full object-cover" />
							<div
								class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent"
							></div>
							<div class="absolute right-6 bottom-6 left-6">
								<h1 class="text-3xl font-bold text-white">{detail.name}</h1>
								<p class="mt-2 text-sm text-gray-200">
									AppID: {detail.appid ?? '-'} | GameID: {detail.id}
								</p>
							</div>
						</div>
					{/if}

					<div class="p-6">
						<div class="flex flex-wrap items-start justify-between gap-4">
							<div>
								{#if !heroSrc}
									<h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
										{detail.name}
									</h1>
									<p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
										AppID: {detail.appid ?? '-'} | GameID: {detail.id}
									</p>
								{/if}
							</div>
							<span
								class={`rounded-full px-3 py-1 text-xs ${
									detail.is_installed
										? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
										: 'bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300'
								}`}
							>
								{detail.is_installed ? t('已安装', 'Installed') : t('未安装', 'Not Installed')}
							</span>
						</div>

						<div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
							<div class="subtle-panel p-4">
								<p class="text-xs text-blue-700 dark:text-blue-300">
									{t('累计游玩', 'Total Playtime')}
								</p>
								<p class="mt-2 text-xl font-semibold text-blue-900 dark:text-blue-100">
									{detail.total_playtime_formatted}
								</p>
							</div>
							<div class="subtle-panel p-4">
								<p class="text-xs text-purple-700 dark:text-purple-300">
									{t('总会话数', 'Total Sessions')}
								</p>
								<p class="mt-2 text-xl font-semibold text-purple-900 dark:text-purple-100">
									{detail.session_count}
								</p>
							</div>
							<div class="subtle-panel p-4">
								<p class="text-xs text-amber-700 dark:text-amber-300">
									{t('平均会话', 'Average Session')}
								</p>
								<p class="mt-2 text-xl font-semibold text-amber-900 dark:text-amber-100">
									{detail.average_session_formatted}
								</p>
							</div>
							<div class="subtle-panel p-4">
								<p class="text-xs text-slate-700 dark:text-slate-300">
									{t('最后游玩', 'Last Played')}
								</p>
								<p class="mt-2 text-sm font-semibold text-slate-900 dark:text-slate-100">
									{formatDateTime(detail.last_played_at)}
								</p>
							</div>
						</div>

						<div class="mt-6 grid gap-4 md:grid-cols-2">
							<div
								class="rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#101522]"
							>
								<p class="text-xs tracking-wide text-gray-500 uppercase dark:text-gray-400">
									{t('安装路径', 'Install Path')}
								</p>
								<p class="mt-2 text-sm break-all text-gray-800 dark:text-gray-200">
									{detail.install_path ?? '-'}
								</p>
							</div>
							<div
								class="rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#101522]"
							>
								<p class="text-xs tracking-wide text-gray-500 uppercase dark:text-gray-400">
									{t('更新时间', 'Updated At')}
								</p>
								<p class="mt-2 text-sm text-gray-800 dark:text-gray-200">
									{formatDateTime(detail.updated_at)}
								</p>
							</div>
						</div>
					</div>
				</article>

				<article class="surface-card p-6">
					<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
						{t('最近会话', 'Recent Sessions')}
					</h2>

					{#if detail.recent_sessions.length === 0}
						<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">
							{t('暂无会话数据', 'No session data')}
						</p>
					{:else}
						<div
							class="mt-4 overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700"
						>
							<table class="min-w-full divide-y divide-gray-200 text-sm dark:divide-gray-700">
								<thead class="bg-gray-50 dark:bg-[#101522]">
									<tr>
										<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
											{t('会话ID', 'Session ID')}
										</th>
										<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
											{t('开始时间', 'Start Time')}
										</th>
										<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
											{t('结束时间', 'End Time')}
										</th>
										<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">
											{t('时长', 'Duration')}
										</th>
										<th class="px-4 py-3 text-right font-medium text-gray-600 dark:text-gray-300">
											{t('操作', 'Actions')}
										</th>
									</tr>
								</thead>
								<tbody
									class="divide-y divide-gray-100 bg-white dark:divide-gray-800 dark:bg-[#151926]"
								>
									{#each detail.recent_sessions as session (session.session_id)}
										<tr>
											<td class="px-4 py-3 text-gray-800 dark:text-gray-200">
												#{session.session_id}
											</td>
											<td class="px-4 py-3 text-gray-600 dark:text-gray-300">
												{formatDateTime(session.start_time)}
											</td>
											<td class="px-4 py-3 text-gray-600 dark:text-gray-300">
												{formatDateTime(session.end_time)}
											</td>
											<td class="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">
												{session.formatted}
											</td>
											<td class="px-4 py-3 text-right">
												<button
													type="button"
													onclick={() => deleteSession(session.session_id)}
													class="rounded-md border border-red-200 px-3 py-1.5 text-xs font-medium text-red-700 hover:bg-red-50 disabled:opacity-60 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950/30"
													disabled={actionLoading}
												>
													{t('删除', 'Delete')}
												</button>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					{/if}
				</article>
			</div>
		{/if}
	</div>
</section>

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

<ConfirmDialog
	open={noticeDialog.open}
	title={noticeDialog.title}
	message={noticeDialog.message}
	confirmText={noticeDialog.confirmText}
	showCancel={false}
	onConfirm={closeNotice}
	onCancel={closeNotice}
/>
