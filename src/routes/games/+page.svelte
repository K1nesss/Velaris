<script lang="ts">
  import { onMount } from 'svelte';
  import { getGamesList, type GameListItem } from '$lib/api';

  let loading = $state(true);
  let refreshing = $state(false);
  let search = $state('');
  let installedOnly = $state(false);
  let errorMessage = $state('');
  let games = $state<GameListItem[]>([]);

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

  async function loadGames(showLoading: boolean) {
    if (showLoading) {
      loading = true;
    } else {
      refreshing = true;
    }

    errorMessage = '';
    try {
      games = await getGamesList(search, installedOnly, 500);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  onMount(() => {
    loadGames(true);
  });
</script>

<section class="space-y-6">
  <div class="rounded-2xl border border-gray-200 bg-white/90 p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
    <div class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <p class="text-xs font-medium uppercase tracking-[0.24em] text-gray-500 dark:text-gray-400">Games</p>
        <h2 class="mt-2 text-2xl font-semibold text-gray-900 dark:text-gray-100">游戏库</h2>
      </div>

      <div class="flex w-full flex-wrap items-center gap-3 md:w-auto md:flex-nowrap">
        <input
          type="text"
          bind:value={search}
          placeholder="搜索游戏名称"
          class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-900 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100 md:w-64"
          onkeydown={(event) => {
            if (event.key === 'Enter') {
              loadGames(false);
            }
          }}
        />

        <label class="flex h-10 items-center gap-2 rounded-lg border border-gray-300 px-3 text-sm text-gray-700 dark:border-gray-700 dark:text-gray-200">
          <input type="checkbox" bind:checked={installedOnly} class="h-4 w-4" />
          仅已安装
        </label>

        <button
          class="relative inline-flex h-10 w-24 items-center justify-center rounded-lg bg-blue-600 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
          disabled={loading || refreshing}
          onclick={() => loadGames(false)}
        >
          <span class={`transition-opacity ${refreshing ? 'opacity-0' : 'opacity-100'}`}>刷新</span>
          <span class={`absolute inset-0 flex items-center justify-center transition-opacity ${refreshing ? 'opacity-100' : 'opacity-0'}`}>
            刷新中...
          </span>
        </button>
      </div>
    </div>

    {#if errorMessage}
      <p class="mt-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300">
        加载失败：{errorMessage}
      </p>
    {/if}
  </div>

  {#if loading}
    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      {#each [1, 2, 3, 4, 5, 6] as row (row)}
        <article class="rounded-2xl border border-gray-200 bg-white/90 p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
          <div class="h-5 w-2/3 rounded-md skeleton-shimmer"></div>
          <div class="mt-3 h-4 w-1/3 rounded-md skeleton-shimmer"></div>
          <div class="mt-6 grid grid-cols-2 gap-3">
            <div class="h-12 rounded-xl skeleton-shimmer"></div>
            <div class="h-12 rounded-xl skeleton-shimmer"></div>
          </div>
        </article>
      {/each}
    </div>
  {:else if games.length === 0}
    <article class="rounded-2xl border border-dashed border-gray-300 bg-white/80 p-10 text-center dark:border-gray-700 dark:bg-[#151926]">
      <p class="text-sm text-gray-500 dark:text-gray-400">没有找到符合条件的游戏</p>
    </article>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
      {#each games as game (game.id)}
        <a
          href={`/games/${game.id}`}
          class="block rounded-2xl border border-gray-200 bg-white/90 p-5 shadow-sm transition hover:-translate-y-0.5 hover:border-blue-300 hover:shadow-md dark:border-gray-800 dark:bg-[#151926] dark:hover:border-blue-700"
        >
          <div class="flex items-start justify-between gap-3">
            <h3 class="line-clamp-2 text-lg font-semibold text-gray-900 dark:text-gray-100">{game.name}</h3>
            <span
              class={`shrink-0 rounded-full px-2.5 py-1 text-xs ${
                game.is_installed
                  ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
                  : 'bg-gray-200 text-gray-700 dark:bg-gray-800 dark:text-gray-300'
              }`}
            >
              {game.is_installed ? '已安装' : '未安装'}
            </span>
          </div>

          <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">AppID: {game.appid ?? '-'}</p>

          <div class="mt-5 grid grid-cols-2 gap-3">
            <div class="rounded-xl bg-blue-50 p-3 dark:bg-blue-950/20">
              <p class="text-xs text-blue-700 dark:text-blue-300">累计时长</p>
              <p class="mt-1 text-sm font-semibold text-blue-900 dark:text-blue-100">{game.total_playtime_formatted}</p>
            </div>
            <div class="rounded-xl bg-purple-50 p-3 dark:bg-purple-950/20">
              <p class="text-xs text-purple-700 dark:text-purple-300">会话数</p>
              <p class="mt-1 text-sm font-semibold text-purple-900 dark:text-purple-100">{game.session_count}</p>
            </div>
          </div>

          <p class="mt-4 text-xs text-gray-500 dark:text-gray-400">最后游玩：{formatDateTime(game.last_played_at)}</p>
        </a>
      {/each}
    </div>
  {/if}
</section>