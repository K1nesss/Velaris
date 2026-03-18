<script lang="ts">
	import { onMount } from 'svelte';
	import { getGameDetail, type GameDetail } from '$lib/api';

	let { data } = $props<{ data: { gameId: number } }>();

	let loading = $state(true);
	let errorMessage = $state('');
	let detail = $state<GameDetail | null>(null);

	function formatDateTime(timestamp: number | null) {
		if (!timestamp) {
			return '-';
		}
		return new Intl.DateTimeFormat('zh-CN', {
			year: 'numeric',
			month: '2-digit',
			day: '2-digit',
			hour: '2-digit',
			minute: '2-digit',
		}).format(new Date(timestamp * 1000));
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

	onMount(() => {
		loadDetail();
	});
</script>

<section class="space-y-6">
	<div class="flex items-center justify-between gap-3">
		<a
			href="/games"
			class="inline-flex items-center rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-700 transition hover:bg-gray-100 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
		>
			返回游戏库
		</a>
		<button
			onclick={loadDetail}
			class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-blue-700"
		>
			刷新
		</button>
	</div>

	{#if loading}
		<article class="rounded-2xl border border-gray-200 bg-white/90 p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
			<div class="h-8 w-64 rounded-md skeleton-shimmer"></div>
			<div class="mt-3 h-4 w-40 rounded-md skeleton-shimmer"></div>
			<div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
				{#each [1, 2, 3, 4] as row (row)}
					<div class="h-20 rounded-xl skeleton-shimmer"></div>
				{/each}
			</div>
		</article>
	{:else if errorMessage}
		<article class="rounded-2xl border border-red-200 bg-red-50 p-6 text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300">
			加载失败：{errorMessage}
		</article>
	{:else if detail}
		<div class="space-y-6">
			<article class="rounded-2xl border border-gray-200 bg-white/90 p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
				<div class="flex flex-wrap items-start justify-between gap-4">
					<div>
						<h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">{detail.name}</h1>
						<p class="mt-2 text-sm text-gray-500 dark:text-gray-400">AppID: {detail.appid ?? '-'} | GameID: {detail.id}</p>
					</div>
					<span
						class={`rounded-full px-3 py-1 text-xs ${
							detail.is_installed
								? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
								: 'bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300'
						}`}
					>
						{detail.is_installed ? '已安装' : '未安装'}
					</span>
				</div>

				<div class="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
					<div class="rounded-xl bg-blue-50 p-4 dark:bg-blue-950/20">
						<p class="text-xs text-blue-700 dark:text-blue-300">累计游玩</p>
						<p class="mt-2 text-xl font-semibold text-blue-900 dark:text-blue-100">{detail.total_playtime_formatted}</p>
					</div>
					<div class="rounded-xl bg-purple-50 p-4 dark:bg-purple-950/20">
						<p class="text-xs text-purple-700 dark:text-purple-300">总会话数</p>
						<p class="mt-2 text-xl font-semibold text-purple-900 dark:text-purple-100">{detail.session_count}</p>
					</div>
					<div class="rounded-xl bg-amber-50 p-4 dark:bg-amber-950/20">
						<p class="text-xs text-amber-700 dark:text-amber-300">平均会话</p>
						<p class="mt-2 text-xl font-semibold text-amber-900 dark:text-amber-100">{detail.average_session_formatted}</p>
					</div>
					<div class="rounded-xl bg-slate-50 p-4 dark:bg-slate-900">
						<p class="text-xs text-slate-700 dark:text-slate-300">最后游玩</p>
						<p class="mt-2 text-sm font-semibold text-slate-900 dark:text-slate-100">{formatDateTime(detail.last_played_at)}</p>
					</div>
				</div>

				<div class="mt-6 grid gap-4 md:grid-cols-2">
					<div class="rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#101522]">
						<p class="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">安装路径</p>
						<p class="mt-2 break-all text-sm text-gray-800 dark:text-gray-200">{detail.install_path ?? '-'}</p>
					</div>
					<div class="rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#101522]">
						<p class="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">更新时间</p>
						<p class="mt-2 text-sm text-gray-800 dark:text-gray-200">{formatDateTime(detail.updated_at)}</p>
					</div>
				</div>
			</article>

			<article class="rounded-2xl border border-gray-200 bg-white/90 p-6 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
				<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">最近会话</h2>

				{#if detail.recent_sessions.length === 0}
					<p class="mt-4 text-sm text-gray-500 dark:text-gray-400">暂无会话数据</p>
				{:else}
					<div class="mt-4 overflow-hidden rounded-xl border border-gray-200 dark:border-gray-700">
						<table class="min-w-full divide-y divide-gray-200 text-sm dark:divide-gray-700">
							<thead class="bg-gray-50 dark:bg-[#101522]">
								<tr>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">会话ID</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">开始时间</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">结束时间</th>
									<th class="px-4 py-3 text-left font-medium text-gray-600 dark:text-gray-300">时长</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-gray-100 bg-white dark:divide-gray-800 dark:bg-[#151926]">
								{#each detail.recent_sessions as session (session.session_id)}
									<tr>
										<td class="px-4 py-3 text-gray-800 dark:text-gray-200">#{session.session_id}</td>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">{formatDateTime(session.start_time)}</td>
										<td class="px-4 py-3 text-gray-600 dark:text-gray-300">{formatDateTime(session.end_time)}</td>
										<td class="px-4 py-3 font-medium text-gray-900 dark:text-gray-100">{session.formatted}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</article>
		</div>
	{/if}
</section>
