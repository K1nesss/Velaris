<script lang="ts">
  import { onMount } from 'svelte';
  import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';

  const CURRENT_PLAYING_POLL_INTERVAL_MS = 2000;
  const DEFAULT_SESSION_COMPLETION_POLL_INTERVAL_MS = 5000;

  import {
    getDashboardSnapshot,
    getDashboardCurrentPlaying,
    getDashboardRecentSessions,
    type DashboardSnapshot,
    type CurrentPlayingGame,
    type DailyChartItem,
    type DonutChartItem,
    type RecentSessionItem,
  } from '$lib/api';
  import CurrentPlayingCard from './components/CurrentPlayingCard.svelte';
  import RecentSessionsList from './components/RecentSessionsList.svelte';
  import DailyChart from './components/DailyChart.svelte';
  import DonutChart from './components/DonutChart.svelte';

  let todayPlaytime = $state('2h 15m');
  let weekPlaytime = $state('9h 42m');
  let initialLoading = $state(true);
  let currentPlaying = $state<CurrentPlayingGame | null>(null);
  let recentSessions = $state<RecentSessionItem[]>([]);
  let dailyChartItems = $state<DailyChartItem[]>([]);
  let donutItems = $state<DonutChartItem[]>([]);
  let dashboardRefreshing = false;
  let currentPlayingRefreshing = false;
  let sessionCompletionRefreshing = false;
  let metricsRefreshing = false;
  let latestCompletedSessionId = $state<number | null>(null);
  let language = $state<AppLanguage>('zh-CN');

  function t(zh: string, en: string) {
    return language === 'zh-CN' ? zh : en;
  }

  const DASHBOARD_CACHE_KEY = 'dashboard_snapshot_cache_v1';

  function applySnapshot(snapshot: DashboardSnapshot) {
    todayPlaytime = snapshot.today.formatted;
    weekPlaytime = snapshot.week.formatted;
    currentPlaying = snapshot.current_playing;
    recentSessions = snapshot.recent_sessions;
    dailyChartItems = snapshot.daily_chart;
    donutItems = snapshot.donut;
    latestCompletedSessionId = getLatestCompletedSessionId(snapshot.recent_sessions);
  }

  function cacheSnapshot(snapshot: DashboardSnapshot) {
    try {
      localStorage.setItem(DASHBOARD_CACHE_KEY, JSON.stringify(snapshot));
    } catch (error) {
      console.warn('Failed to cache dashboard snapshot', error);
    }
  }

  function hydrateFromCache() {
    try {
      const raw = localStorage.getItem(DASHBOARD_CACHE_KEY);
      if (!raw) {
        return false;
      }

      const snapshot = JSON.parse(raw) as DashboardSnapshot;
      applySnapshot(snapshot);
      initialLoading = false;
      return true;
    } catch (error) {
      console.warn('Failed to hydrate dashboard cache', error);
      return false;
    }
  }

  function getLatestCompletedSessionId(sessions: RecentSessionItem[]) {
    return sessions.find((session) => session.end_time !== null)?.session_id ?? null;
  }

  async function refreshMetrics(showLoading = false) {
    if (metricsRefreshing) {
      return;
    }

    metricsRefreshing = true;
    if (showLoading) {
      initialLoading = true;
    }

    try {
      const snapshot = await getDashboardSnapshot(7, 10, 8);
      todayPlaytime = snapshot.today.formatted;
      weekPlaytime = snapshot.week.formatted;
      dailyChartItems = snapshot.daily_chart;
      donutItems = snapshot.donut;
      cacheSnapshot({
        ...snapshot,
        current_playing: currentPlaying,
        recent_sessions: recentSessions,
      });
    } catch (error) {
      console.error('Failed to refresh dashboard metrics', error);
    } finally {
      metricsRefreshing = false;
      if (showLoading) {
        initialLoading = false;
      }
    }
  }

  async function refreshCurrentPlaying() {
    if (currentPlayingRefreshing) {
      return;
    }

    currentPlayingRefreshing = true;
    try {
      const previous = currentPlaying;
      const current = await getDashboardCurrentPlaying();
      const gameStarted = previous === null && current !== null;

      currentPlaying = current;

      // When a new game is detected as started, update recent session list immediately.
      if (gameStarted) {
        const sessions = await getDashboardRecentSessions(8);
        recentSessions = sessions;
        latestCompletedSessionId = getLatestCompletedSessionId(sessions);
        cacheSnapshot({
          today: { seconds: 0, formatted: todayPlaytime },
          week: { seconds: 0, formatted: weekPlaytime },
          current_playing: current,
          recent_sessions: sessions,
          daily_chart: dailyChartItems,
          donut: donutItems,
        });
      }
    } catch (error) {
      console.error('Failed to refresh current playing game', error);
    } finally {
      currentPlayingRefreshing = false;
    }
  }

  async function refreshWhenSessionCompleted() {
    if (sessionCompletionRefreshing) {
      return;
    }

    sessionCompletionRefreshing = true;
    try {
      const sessions = await getDashboardRecentSessions(8);
      recentSessions = sessions;

      const newestCompletedSessionId = getLatestCompletedSessionId(sessions);
      const hasNewCompletedSession =
        newestCompletedSessionId !== null &&
        latestCompletedSessionId !== null &&
        newestCompletedSessionId !== latestCompletedSessionId;

      latestCompletedSessionId = newestCompletedSessionId;

      // Refresh heavy dashboard metrics only when a new session has ended.
      if (hasNewCompletedSession) {
        await refreshMetrics(false);
      } else {
        cacheSnapshot({
          today: { seconds: 0, formatted: todayPlaytime },
          week: { seconds: 0, formatted: weekPlaytime },
          current_playing: currentPlaying,
          recent_sessions: sessions,
          daily_chart: dailyChartItems,
          donut: donutItems,
        });
      }
    } catch (error) {
      console.error('Failed to check session completion updates', error);
    } finally {
      sessionCompletionRefreshing = false;
    }
  }

  async function loadDashboardData(showLoading = false) {
    if (dashboardRefreshing) {
      return;
    }

    dashboardRefreshing = true;
    if (showLoading) {
      initialLoading = true;
    }

    try {
      const snapshot = await getDashboardSnapshot(7, 10, 8);
      applySnapshot(snapshot);
      cacheSnapshot(snapshot);
    } catch (error) {
      console.error('Failed to load dashboard data', error);
    } finally {
      dashboardRefreshing = false;
      if (showLoading) {
        initialLoading = false;
      }
    }
  }

  onMount(() => {
    const settings = loadSettingsFromStorage();
    const sessionCompletionPollIntervalMs = Math.max(2, settings.dashboardRefreshSeconds) * 1000;
    language = settings.language;

    const hydrated = hydrateFromCache();
    loadDashboardData(!hydrated);

    const currentPlayingIntervalId = window.setInterval(() => {
      refreshCurrentPlaying();
    }, CURRENT_PLAYING_POLL_INTERVAL_MS);

    const sessionCompletionIntervalId = window.setInterval(() => {
      refreshWhenSessionCompleted();
    }, sessionCompletionPollIntervalMs || DEFAULT_SESSION_COMPLETION_POLL_INTERVAL_MS);

    const handleWindowFocus = () => {
      refreshCurrentPlaying();
      refreshWhenSessionCompleted();
    };

    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        refreshCurrentPlaying();
        refreshWhenSessionCompleted();
      }
    };

    window.addEventListener('focus', handleWindowFocus);
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      window.clearInterval(currentPlayingIntervalId);
      window.clearInterval(sessionCompletionIntervalId);
      window.removeEventListener('focus', handleWindowFocus);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  });
</script>

<section class="space-y-6">
  <div class="grid gap-6 xl:grid-cols-2">
    <div class="min-w-0 space-y-6">
      <div class="grid gap-6 sm:grid-cols-2">
        <div class="relative overflow-hidden rounded-xl border border-cyan-200/50 bg-white p-6 shadow-sm group dark:border-white/10 dark:bg-[#151926]">
          <div class="absolute inset-0 bg-linear-to-r from-cyan-500/10 to-blue-600/10 opacity-50 transition-opacity group-hover:opacity-30 dark:opacity-20"></div>
          <div class="relative z-10 flex items-center justify-between gap-4">
            <div>
              <h3 class="mb-1 text-xs font-medium uppercase tracking-[0.2em] text-cyan-600 dark:text-cyan-400">{t('今日时长', "Today's Playtime")}</h3>
              {#if initialLoading}
                <div class="mt-2 h-10 w-36 rounded-lg skeleton-shimmer sm:h-11"></div>
              {:else}
                <div class="text-3xl font-bold text-cyan-700 dark:text-cyan-200 sm:text-4xl">{todayPlaytime}</div>
              {/if}
            </div>
            <div class="relative z-10 flex h-12 w-12 items-center justify-center rounded-full bg-cyan-500/20 text-gray-500 dark:text-gray-300">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="h-6 w-6"
                aria-hidden="true"
              >
                <circle cx="12" cy="12" r="10" />
                <path d="M12 6v6l4 2" />
              </svg>
            </div>
          </div>
        </div>

        <div class="relative overflow-hidden rounded-xl border border-purple-200/50 bg-white p-6 shadow-sm group dark:border-white/10 dark:bg-[#151926]">
          <div class="absolute inset-0 bg-linear-to-r from-purple-500/10 to-pink-600/10 opacity-50 transition-opacity group-hover:opacity-30 dark:opacity-20"></div>
          <div class="relative z-10 flex items-center justify-between gap-4">
            <div>
              <h3 class="mb-1 text-xs font-medium uppercase tracking-[0.2em] text-purple-600 dark:text-purple-400">{t('本周时长', 'This Week')}</h3>
              {#if initialLoading}
                <div class="mt-2 h-10 w-36 rounded-lg skeleton-shimmer sm:h-11"></div>
              {:else}
                <div class="text-3xl font-bold text-purple-700 dark:text-purple-200 sm:text-4xl">{weekPlaytime}</div>
              {/if}
            </div>
            <div class="relative z-10 flex h-12 w-12 items-center justify-center rounded-full bg-purple-500/20 text-gray-500 dark:text-gray-300">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="h-6 w-6"
                aria-hidden="true"
              >
                <path d="M8 2v4" />
                <path d="M16 2v4" />
                <rect width="18" height="18" x="3" y="4" rx="2" />
                <path d="M3 10h18" />
              </svg>
            </div>
          </div>
        </div>
      </div>

      <CurrentPlayingCard currentPlaying={currentPlaying} loading={initialLoading} language={language} />
      <RecentSessionsList sessions={recentSessions} loading={initialLoading} language={language} />
    </div>

    <div class="min-w-0 space-y-6">
      <DailyChart items={dailyChartItems} loading={initialLoading} language={language} />
      <DonutChart items={donutItems} loading={initialLoading} language={language} />
    </div>
  </div>
</section>