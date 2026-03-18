import { invoke } from '@tauri-apps/api/core';

export type DurationSummary = {
  seconds: number;
  formatted: string;
};

export type DailyChartItem = {
  day: string;
  seconds: number;
  formatted: string;
};

export type CurrentPlayingGame = {
  session_id: number;
  game_id: number;
  game_name: string;
  start_time: number;
  current_duration_seconds: number;
  formatted: string;
};

export type DonutChartItem = {
  game_id: number;
  name: string;
  seconds: number;
  formatted: string;
  percentage: number;
};

export type RecentSessionItem = {
  session_id: number;
  game_id: number;
  game_name: string;
  start_time: number;
  end_time: number | null;
  duration_seconds: number;
  formatted: string;
};

export type DashboardSnapshot = {
  today: DurationSummary;
  week: DurationSummary;
  current_playing: CurrentPlayingGame | null;
  recent_sessions: RecentSessionItem[];
  daily_chart: DailyChartItem[];
  donut: DonutChartItem[];
};

export async function hello() {
  return await invoke<string>('hello');
}

export async function printDatabaseTables() {
  await invoke('print_database_tables');
}

export async function getDashboardTodayTotal() {
  return await invoke<DurationSummary>('dashboard_today_total');
}

export async function getDashboardWeekTotal() {
  return await invoke<DurationSummary>('dashboard_week_total');
}

export async function getDashboardDailyChart(days?: number) {
  return await invoke<DailyChartItem[]>('dashboard_daily_chart', { days });
}

export async function getDashboardCurrentPlaying() {
  return await invoke<CurrentPlayingGame | null>('dashboard_current_playing');
}

export async function getDashboardDonutData(limit?: number) {
  return await invoke<DonutChartItem[]>('dashboard_donut_data', { limit });
}

export async function getDashboardRecentSessions(limit?: number) {
  return await invoke<RecentSessionItem[]>('dashboard_recent_sessions', { limit });
}

export async function getDashboardSnapshot(
  days = 7,
  donutLimit = 10,
  recentLimit = 8,
) {
  return await invoke<DashboardSnapshot>('dashboard_snapshot', {
    days,
    donut_limit: donutLimit,
    recent_limit: recentLimit,
  });
}