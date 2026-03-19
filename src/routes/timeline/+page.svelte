<script lang="ts">
  import { onMount } from 'svelte';
  import { getTimelineSessions, type TimelineSessionItem } from '$lib/api';

  const PAGE_SIZE = 80;

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
  let endedOnly = $state(false);
  let selectedGameId = $state('all');
  let durationRange = $state<DurationRange>('all');
  let showFilterPanel = $state(false);
  let allSessions = $state<TimelineSessionItem[]>([]);
  let groups = $state<TimelineDayGroup[]>([]);
  let hasMore = $state(true);
  let filterPanelRef = $state<HTMLDivElement | null>(null);
  let filterButtonRef = $state<HTMLButtonElement | null>(null);

  const accents: Accent[] = [
    {
      dot: 'border-yellow-500 text-yellow-500',
      panel: 'bg-yellow-950/20',
      chip: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-300',
    },
    {
      dot: 'border-red-500 text-red-500',
      panel: 'bg-red-950/20',
      chip: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300',
    },
    {
      dot: 'border-emerald-500 text-emerald-500',
      panel: 'bg-emerald-950/20',
      chip: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300',
    },
    {
      dot: 'border-indigo-500 text-indigo-500',
      panel: 'bg-indigo-950/20',
      chip: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-300',
    },
    {
      dot: 'border-cyan-500 text-cyan-500',
      panel: 'bg-cyan-950/20',
      chip: 'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300',
    },
  ];

  function getAccent(name: string) {
    const hash = Array.from(name).reduce((acc, char) => acc + char.charCodeAt(0), 0);
    return accents[hash % accents.length];
  }

  function formatDateTime(timestamp: number) {
    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(new Date(timestamp * 1000));
  }

  function formatTimeRange(startTime: number, endTime: number | null) {
    const start = new Intl.DateTimeFormat('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    }).format(new Date(startTime * 1000));

    if (!endTime) {
      return `${start} - 进行中`;
    }

    const end = new Intl.DateTimeFormat('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
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
    const yesterday = new Date();
    yesterday.setDate(today.getDate() - 1);

    const isSameDay =
      target.getFullYear() === today.getFullYear() &&
      target.getMonth() === today.getMonth() &&
      target.getDate() === today.getDate();

    if (isSameDay) {
      return 'Today';
    }

    const isYesterday =
      target.getFullYear() === yesterday.getFullYear() &&
      target.getMonth() === yesterday.getMonth() &&
      target.getDate() === yesterday.getDate();

    if (isYesterday) {
      return 'Yesterday';
    }

    return new Intl.DateTimeFormat('zh-CN', {
      month: 'long',
      day: '2-digit',
      weekday: 'short',
    }).format(target);
  }

  function hasActiveFilters() {
    return endedOnly || selectedGameId !== 'all' || durationRange !== 'all';
  }

  function resetFilters() {
    endedOnly = false;
    selectedGameId = 'all';
    durationRange = 'all';
  }

  function getGameOptions() {
    const map = new Map<number, string>();
    for (const session of allSessions) {
      if (!map.has(session.game_id)) {
        map.set(session.game_id, session.game_name);
      }
    }

    return Array.from(map.entries())
      .map(([id, name]) => ({ id, name }))
      .sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'));
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

      const keyword = search.trim().toLowerCase();
      if (!keyword) {
        return true;
      }

      return (
        item.game_name.toLowerCase().includes(keyword) ||
        String(item.session_id).includes(keyword) ||
        String(item.appid ?? '').includes(keyword)
      );
    });

    const map = new Map<string, TimelineDayGroup>();

    for (const item of filtered) {
      const date = new Date(item.start_time * 1000);
      const key = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`;

      if (!map.has(key)) {
        map.set(key, {
          key,
          label: formatGroupLabel(item.start_time),
          totalSeconds: 0,
          totalFormatted: '0s',
          sessions: [],
        });
      }

      const group = map.get(key)!;
      group.sessions.push(item);
      group.totalSeconds += item.duration_seconds;
    }

    const next = Array.from(map.values());
    for (const group of next) {
      group.totalFormatted = formatDuration(group.totalSeconds);
    }
    groups = next;
  }

  async function loadTimeline(showLoading: boolean) {
    if (showLoading) {
      loading = true;
    } else {
      refreshing = true;
    }

    errorMessage = '';
    try {
      const data = await getTimelineSessions(PAGE_SIZE, 0);
      allSessions = data;
      hasMore = data.length === PAGE_SIZE;
      buildGroups(allSessions);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function loadOlder() {
    if (!hasMore || loadingMore || loading || refreshing) {
      return;
    }

    loadingMore = true;
    try {
      const next = await getTimelineSessions(PAGE_SIZE, allSessions.length);
      allSessions = [...allSessions, ...next];
      hasMore = next.length === PAGE_SIZE;
      buildGroups(allSessions);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
    } finally {
      loadingMore = false;
    }
  }

  onMount(() => {
    loadTimeline(true);

    const handlePointerDown = (event: PointerEvent) => {
      if (!showFilterPanel) {
        return;
      }

      const target = event.target as Node;
      if (filterPanelRef?.contains(target) || filterButtonRef?.contains(target)) {
        return;
      }

      showFilterPanel = false;
    };

    window.addEventListener('pointerdown', handlePointerDown);
    return () => {
      window.removeEventListener('pointerdown', handlePointerDown);
    };
  });

  $effect(() => {
    search;
    endedOnly;
    selectedGameId;
    durationRange;
    buildGroups(allSessions);
  });
</script>

<div class="flex h-full flex-col p-4 md:p-8">
  <div class="mb-6 flex shrink-0 flex-wrap items-center justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Activity Timeline</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400">Track your gaming sessions history</p>
    </div>

    <div class="flex w-full flex-wrap gap-3 md:w-auto md:flex-nowrap">
      <div class="relative w-full md:w-64">
        <input
          class="h-10 w-full rounded-lg border border-gray-300 bg-white pl-10 pr-4 text-sm text-gray-900 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200"
          placeholder="Search sessions..."
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
          class="pointer-events-none absolute left-3 top-2.5 h-5 w-5 text-gray-400"
          aria-hidden="true"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.3-4.3" />
        </svg>
      </div>

      <div class="relative">
        <button
          bind:this={filterButtonRef}
          class={`inline-flex h-10 w-10 items-center justify-center rounded-lg border transition ${
            hasActiveFilters()
              ? 'border-blue-500 bg-blue-50 text-blue-600 dark:border-blue-500 dark:bg-blue-950/30 dark:text-blue-300'
              : 'border-gray-300 bg-white text-gray-500 hover:text-gray-700 dark:border-gray-700 dark:bg-[#151926] dark:hover:text-gray-300'
          }`}
          title="Filter sessions"
          onclick={() => {
            showFilterPanel = !showFilterPanel;
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
            class="absolute right-0 z-20 mt-2 w-72 rounded-xl border border-gray-200 bg-white p-4 shadow-lg dark:border-gray-700 dark:bg-[#151926]"
          >
            <div class="space-y-4">
              <label class="flex cursor-pointer items-center justify-between gap-3 text-sm text-gray-700 dark:text-gray-200">
                <span>仅已结束</span>
                <input type="checkbox" bind:checked={endedOnly} class="h-4 w-4" />
              </label>

              <div class="space-y-2">
                <label for="timeline-game-filter" class="text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">按游戏筛选</label>
                <select
                  id="timeline-game-filter"
                  bind:value={selectedGameId}
                  class="h-9 w-full rounded-lg border border-gray-300 bg-white px-2 text-sm text-gray-800 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100"
                >
                  <option value="all">全部游戏</option>
                  {#each getGameOptions() as option (option.id)}
                    <option value={String(option.id)}>{option.name}</option>
                  {/each}
                </select>
              </div>

              <div class="space-y-2">
                <label for="timeline-duration-filter" class="text-xs font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">按时长区间</label>
                <select
                  id="timeline-duration-filter"
                  bind:value={durationRange}
                  class="h-9 w-full rounded-lg border border-gray-300 bg-white px-2 text-sm text-gray-800 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100"
                >
                  <option value="all">全部时长</option>
                  <option value="lt30m">小于 30 分钟</option>
                  <option value="30to60m">30 分钟 - 1 小时</option>
                  <option value="1to2h">1 小时 - 2 小时</option>
                  <option value="gt2h">大于等于 2 小时</option>
                </select>
              </div>

              <div class="flex items-center justify-between gap-3 pt-1">
                <button
                  class="rounded-lg border border-gray-300 px-3 py-1.5 text-xs text-gray-700 transition hover:bg-gray-100 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
                  onclick={resetFilters}
                >
                  重置
                </button>
                <button
                  class="rounded-lg bg-blue-600 px-3 py-1.5 text-xs font-medium text-white transition hover:bg-blue-700"
                  onclick={() => {
                    showFilterPanel = false;
                  }}
                >
                  应用
                </button>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <button
        class="relative inline-flex h-10 w-24 items-center justify-center rounded-lg bg-blue-600 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
        disabled={loading || refreshing}
        onclick={() => loadTimeline(false)}
      >
        <span class={`transition-opacity ${refreshing ? 'opacity-0' : 'opacity-100'}`}>刷新</span>
        <span class={`absolute inset-0 flex items-center justify-center transition-opacity ${refreshing ? 'opacity-100' : 'opacity-0'}`}>
          刷新中...
        </span>
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto pr-2 no-scrollbar min-h-0">
    {#if errorMessage}
      <p class="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300">
        加载失败：{errorMessage}
      </p>
    {/if}

    {#if loading}
      <div class="mx-auto max-w-4xl space-y-6 pb-8">
        {#each [1, 2, 3] as row (row)}
          <article class="rounded-2xl border border-gray-200 bg-white/90 p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
            <div class="h-5 w-48 rounded-md skeleton-shimmer"></div>
            <div class="mt-4 space-y-3">
              {#each [1, 2] as item (item)}
                <div class="h-24 rounded-xl skeleton-shimmer"></div>
              {/each}
            </div>
          </article>
        {/each}
      </div>
    {:else if groups.length === 0}
      <article class="mx-auto max-w-4xl rounded-2xl border border-dashed border-gray-300 bg-white/80 p-10 text-center dark:border-gray-700 dark:bg-[#151926]">
        <p class="text-sm text-gray-500 dark:text-gray-400">暂无符合筛选条件的时间线数据</p>
      </article>
    {:else}
      <div class="mx-auto max-w-4xl space-y-10 pb-8">
        {#each groups as group (group.key)}
          <div class="relative">
            <div class="mb-4 flex items-center py-2">
              <span class="rounded-full border border-blue-200 bg-blue-100 px-3 py-1 text-xs font-bold uppercase tracking-wider text-blue-700 dark:border-blue-700/50 dark:bg-blue-900/30 dark:text-blue-300">
                {group.label}
              </span>
              <div class="ml-4 h-px flex-1 bg-gray-300 dark:bg-gray-800"></div>
              <span class="ml-4 text-xs text-gray-500 dark:text-gray-500">{group.totalFormatted} Total</span>
            </div>

            <div class="space-y-4">
              {#each group.sessions as session, index (session.session_id)}
                {@const accent = getAccent(session.game_name)}
                <div class="relative pl-10">
                  <div class="absolute left-0 top-0 bottom-0 flex flex-col items-center">
                    <div class={`z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 bg-white dark:bg-[#151926] ${accent.dot}`}>
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

                  <details class="group overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm transition-all hover:shadow-md dark:border-gray-800 dark:bg-[#151926] dark:hover:border-gray-700">
                    <summary class="flex list-none cursor-pointer select-none items-center justify-between p-4">
                      <div class="flex items-center space-x-4">
                        <div class={`relative flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg ${accent.panel}`}>
                          <span class="text-base font-bold text-white/90">{session.game_name.slice(0, 1).toUpperCase()}</span>
                          <div class={`absolute bottom-0 left-0 right-0 h-1 ${accent.chip.split(' ')[0]}`}></div>
                        </div>
                        <div>
                          <h3 class="text-lg font-bold text-gray-900 dark:text-white">{session.game_name}</h3>
                          <div class="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
                            <span class="rounded border border-gray-200 bg-gray-100 px-1.5 py-0.5 text-gray-600 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300">Game #{session.game_id}</span>
                            <span>•</span>
                            <span class="font-mono">{formatTimeRange(session.start_time, session.end_time)}</span>
                          </div>
                        </div>
                      </div>

                      <div class="flex items-center space-x-6">
                        <div class="text-right">
                          <div class="font-mono text-2xl font-bold text-gray-900 dark:text-gray-100">{session.formatted}</div>
                          {#if session.is_active}
                            <div class="mt-0.5 text-xs font-medium text-emerald-500">Running</div>
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
                          class="h-5 w-5 text-gray-400 transition-transform group-open:rotate-180"
                          aria-hidden="true"
                        >
                          <path d="m6 9 6 6 6-6"></path>
                        </svg>
                      </div>
                    </summary>

                    <div class="border-t border-gray-100 px-4 pb-4 pt-0 dark:border-gray-800/50">
                      <div class="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
                        <div class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]">
                          <span class="mb-1 block text-xs uppercase tracking-wide text-gray-500">Session ID</span>
                          <div class="text-sm font-medium text-gray-800 dark:text-gray-300">#{session.session_id}</div>
                        </div>
                        <div class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]">
                          <span class="mb-1 block text-xs uppercase tracking-wide text-gray-500">Started At</span>
                          <div class="text-sm font-medium text-gray-800 dark:text-gray-300">{formatDateTime(session.start_time)}</div>
                        </div>
                        <div class="rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]">
                          <span class="mb-1 block text-xs uppercase tracking-wide text-gray-500">Ended At</span>
                          <div class="text-sm font-medium text-gray-800 dark:text-gray-300">{session.end_time ? formatDateTime(session.end_time) : 'Still Running'}</div>
                        </div>
                      </div>

                      <div class="mt-4 flex items-center justify-between">
                        <a href={`/games/${session.game_id}`} class="text-sm font-medium text-blue-600 transition-colors hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300">
                          查看游戏详情
                        </a>
                        <span class={`rounded-full px-2.5 py-1 text-xs ${accent.chip}`}>AppID: {session.appid ?? '-'}</span>
                      </div>
                    </div>
                  </details>
                </div>
              {/each}
            </div>
          </div>
        {/each}

        <div class="flex justify-center pb-2 pt-2">
          <button
            class="rounded-full border border-gray-200 bg-white px-6 py-2 text-sm font-medium text-gray-600 transition-colors hover:text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-800 dark:bg-[#151926] dark:text-gray-400 dark:hover:text-white"
            disabled={!hasMore || loadingMore}
            onclick={loadOlder}
          >
            {loadingMore ? 'Loading...' : hasMore ? 'Load Older Sessions' : 'No More Sessions'}
          </button>
        </div>
      </div>
    {/if}
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