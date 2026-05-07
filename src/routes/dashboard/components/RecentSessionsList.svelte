<script lang="ts">
	import type { RecentSessionItem } from '$lib/api';
	import type { AppLanguage } from '$lib/settings';

	let {
		sessions = [],
		loading = false,
		language = 'zh-CN'
	} = $props<{
		sessions?: RecentSessionItem[];
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
</script>

<article class="surface-card flex h-96 min-h-96 flex-col overflow-hidden p-5">
	<div class="flex items-center justify-between gap-3">
		<div>
			<p class="eyebrow">{t('最近会话', 'Recent Sessions')}</p>
			<!-- <h3 class="mt-2 text-lg font-semibold text-gray-900 dark:text-gray-100">最近游玩记录</h3> -->
		</div>
	</div>

	{#if loading}
		<div class="mt-4 min-h-0 flex-1 space-y-2 overflow-hidden pr-1">
			{#each [1, 2, 3] as row (row)}
				<div class="subtle-panel p-3">
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0 flex-1 space-y-2">
							<div class="skeleton-shimmer h-4 w-2/3 rounded-md"></div>
							<div class="skeleton-shimmer h-3 w-1/2 rounded-md"></div>
						</div>
						<div class="skeleton-shimmer h-5 w-16 rounded-full"></div>
					</div>
					<div class="mt-2 flex items-center justify-between gap-3">
						<div class="skeleton-shimmer h-3 w-16 rounded-md"></div>
						<div class="skeleton-shimmer h-3 w-24 rounded-md"></div>
					</div>
				</div>
			{/each}
		</div>
	{:else if sessions.length > 0}
		<div class="mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
			{#each sessions as session (session.session_id)}
				<div class="subtle-panel p-3">
					<div class="flex items-start justify-between gap-3">
						<div class="min-w-0">
							<p class="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">
								{session.game_name}
							</p>
							<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
								{t('开始于', 'Started at')}
								{formatDateTime(session.start_time)}
							</p>
						</div>
						<span
							class="shrink-0 rounded-full bg-slate-200 px-2.5 py-1 text-xs text-slate-700 dark:bg-slate-800 dark:text-slate-300"
						>
							{session.formatted}
						</span>
					</div>

					<div
						class="mt-2 flex items-center justify-between gap-3 text-xs text-gray-500 dark:text-gray-400"
					>
						<span>Game #{session.game_id}</span>
						<span
							>{session.end_time
								? `${t('结束于', 'Ended at')} ${formatDateTime(session.end_time)}`
								: t('仍在进行中', 'Still running')}</span
						>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="empty-state mt-5 flex flex-1 items-center justify-center px-4 py-8">
			<p class="text-sm text-gray-500 dark:text-gray-400">
				{t('还没有可展示的最近会话', 'No recent sessions to display')}
			</p>
		</div>
	{/if}
</article>
