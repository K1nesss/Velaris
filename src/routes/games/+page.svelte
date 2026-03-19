<script lang="ts">
  import { onMount } from 'svelte';
  import { getGamesList, type GameListItem } from '$lib/api';
  import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';

  let loading = $state(true);
  let refreshing = $state(false);
  let search = $state('');
  let installedOnly = $state(false);
  let errorMessage = $state('');
  let games = $state<GameListItem[]>([]);
  let language = $state<AppLanguage>('zh-CN');
  let gamesRequestId = 0;

  const GAMES_CACHE_KEY = 'games_list_cache_v1';

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
      minute: '2-digit',
    }).format(new Date(timestamp * 1000));
  }

  function buildGamesCacheKey() {
    return `${search.trim().toLowerCase()}|${installedOnly ? '1' : '0'}`;
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
          items,
        }),
      );
    } catch {
      // Ignore cache failures.
    }
  }

  async function loadGames(showLoading: boolean) {
    const requestId = ++gamesRequestId;

    if (showLoading) {
      loading = true;
    } else {
      refreshing = true;
    }

    errorMessage = '';
    try {
      const next = await getGamesList(search, installedOnly, 500);
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
      }
    }
  }

  onMount(() => {
    const settings = loadSettingsFromStorage();
    language = settings.language;

    const hydrated = hydrateGamesFromCache();
    loadGames(!hydrated);
  });
</script>

<div class="flex h-full flex-col p-4 md:p-8">
  <div class="mb-6 flex shrink-0 flex-wrap items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{t('游戏库', 'Games Library')}</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400">{t('浏览并追踪你的游戏收藏', 'Browse and track your game collection')}</p>
    </div>

    <div class="flex w-full flex-wrap gap-3 md:w-auto md:flex-nowrap">
      <input
        type="text"
        bind:value={search}
        placeholder={t('搜索游戏名称', 'Search game name')}
        class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-900 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200 md:w-64"
        onkeydown={(event) => {
          if (event.key === 'Enter') {
            loadGames(false);
          }
        }}
      />

      <label class="flex h-10 items-center gap-2 rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200">
        <input type="checkbox" bind:checked={installedOnly} class="h-4 w-4" />
        {t('仅已安装', 'Installed Only')}
      </label>

      <button
        class="relative inline-flex h-10 w-24 items-center justify-center rounded-lg bg-blue-600 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
        disabled={loading || refreshing}
        onclick={() => loadGames(false)}
      >
        <span class={`transition-opacity ${refreshing ? 'opacity-0' : 'opacity-100'}`}>{t('刷新', 'Refresh')}</span>
        <span class={`absolute inset-0 flex items-center justify-center transition-opacity ${refreshing ? 'opacity-100' : 'opacity-0'}`}>
          {t('刷新中...', 'Refreshing...')}
        </span>
      </button>
    </div>
  </div>

  {#if errorMessage}
    <p class="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300">
      {t('加载失败：', 'Load failed: ')}{errorMessage}
    </p>
  {/if}

  <div class="space-y-6">
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
        <p class="text-sm text-gray-500 dark:text-gray-400">{t('没有找到符合条件的游戏', 'No matching games found')}</p>
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
              {game.is_installed ? t('已安装', 'Installed') : t('未安装', 'Not Installed')}
            </span>
          </div>

          <p class="mt-2 text-xs text-gray-500 dark:text-gray-400">AppID: {game.appid ?? '-'}</p>

          <div class="mt-5 grid grid-cols-2 gap-3">
            <div class="rounded-xl bg-blue-50 p-3 dark:bg-blue-950/20">
              <p class="text-xs text-blue-700 dark:text-blue-300">{t('累计时长', 'Total Playtime')}</p>
              <p class="mt-1 text-sm font-semibold text-blue-900 dark:text-blue-100">{game.total_playtime_formatted}</p>
            </div>
            <div class="rounded-xl bg-purple-50 p-3 dark:bg-purple-950/20">
              <p class="text-xs text-purple-700 dark:text-purple-300">{t('会话数', 'Sessions')}</p>
              <p class="mt-1 text-sm font-semibold text-purple-900 dark:text-purple-100">{game.session_count}</p>
            </div>
          </div>

          <p class="mt-4 text-xs text-gray-500 dark:text-gray-400">{t('最后游玩：', 'Last Played: ')}{formatDateTime(game.last_played_at)}</p>
        </a>
      {/each}
      </div>
    {/if}
  </div>
</div>