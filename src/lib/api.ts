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

export type GameListItem = {
	id: number;
	appid: number | null;
	name: string;
	cover_path: string | null;
	icon_path: string | null;
	hero_path: string | null;
	is_installed: boolean;
	is_hidden: boolean;
	total_playtime_seconds: number;
	total_playtime_formatted: string;
	last_played_at: number | null;
	session_count: number;
};

export type GameDetailSession = {
	session_id: number;
	start_time: number;
	end_time: number | null;
	duration_seconds: number;
	formatted: string;
};

export type GameDetail = {
	id: number;
	appid: number | null;
	name: string;
	install_path: string | null;
	cover_path: string | null;
	hero_path: string | null;
	is_installed: boolean;
	is_hidden: boolean;
	created_at: number;
	updated_at: number;
	total_playtime_seconds: number;
	total_playtime_formatted: string;
	last_played_at: number | null;
	session_count: number;
	average_session_seconds: number;
	average_session_formatted: string;
	recent_sessions: GameDetailSession[];
};

export type IgnoredGameItem = {
	appid: number;
	name: string;
	ignored_at: number;
	reason: string | null;
};

export type HiddenGameItem = {
	id: number;
	appid: number | null;
	name: string;
	updated_at: number;
};

export type CreateManualGameRequest = {
	name: string;
	executable_path: string;
	icon_path?: string | null;
	cover_path?: string | null;
	hero_path?: string | null;
};

export type TimelineSessionItem = {
	session_id: number;
	game_id: number;
	appid: number | null;
	game_name: string;
	icon_path: string | null;
	start_time: number;
	end_time: number | null;
	duration_seconds: number;
	formatted: string;
	is_active: boolean;
};

export type SteamIconSyncSummary = {
	matched_games: number;
	downloaded_icons: number;
	downloaded_covers: number;
	downloaded_heroes: number;
	updated_games: number;
};

export type AnalyticsSummary = {
	total_seconds: number;
	total_formatted: string;
	session_count: number;
	average_session_seconds: number;
	average_session_formatted: string;
	active_days: number;
	longest_session_seconds: number;
	longest_session_formatted: string;
};

export type AnalyticsDailyItem = {
	day: string;
	seconds: number;
	formatted: string;
};

export type AnalyticsHourlyItem = {
	hour: number;
	label: string;
	seconds: number;
	formatted: string;
};

export type AnalyticsTopGameItem = {
	game_id: number;
	name: string;
	seconds: number;
	formatted: string;
	session_count: number;
	percentage: number;
	last_played_at: number | null;
};

export type AnalyticsWeekdayItem = {
	weekday: number;
	label: string;
	seconds: number;
	formatted: string;
	sessions: number;
};

export type AnalyticsSnapshot = {
	summary: AnalyticsSummary;
	daily: AnalyticsDailyItem[];
	hourly: AnalyticsHourlyItem[];
	top_games: AnalyticsTopGameItem[];
	weekday_breakdown: AnalyticsWeekdayItem[];
};

export async function hello() {
	return await invoke<string>('hello');
}

export async function printDatabaseTables() {
	await invoke('print_database_tables');
}

export async function exportDatabase() {
	return await invoke<string>('export_database');
}

export async function importDatabase(base64Data: string) {
	await invoke('import_database', {
		base64_data: base64Data
	});
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

export async function getDashboardSnapshot(days = 7, donutLimit = 10, recentLimit = 8) {
	return await invoke<DashboardSnapshot>('dashboard_snapshot', {
		days,
		donut_limit: donutLimit,
		recent_limit: recentLimit
	});
}

export async function getGamesList(search?: string, installedOnly?: boolean, limit = 200) {
	return await invoke<GameListItem[]>('games_list', {
		search: search && search.trim().length > 0 ? search.trim() : null,
		installed_only: installedOnly ?? null,
		limit
	});
}

export async function getGameDetail(id: number, recentLimit = 20) {
	return await invoke<GameDetail>('game_detail', {
		id,
		recent_limit: recentLimit
	});
}

export async function createManualGame(request: CreateManualGameRequest) {
	return await invoke<number>('create_manual_game', { request });
}

export async function hideGame(gameId: number) {
	await invoke('hide_game', { gameId });
}

export async function deleteGame(gameId: number) {
	await invoke('delete_game', { gameId });
}

export async function deleteGameSession(sessionId: number) {
	await invoke('delete_game_session', { sessionId });
}

export async function getHiddenGamesList() {
	return await invoke<HiddenGameItem[]>('hidden_games_list');
}

export async function restoreHiddenGame(gameId: number) {
	await invoke('restore_hidden_game', { gameId });
}

export async function getIgnoredGamesList() {
	return await invoke<IgnoredGameItem[]>('ignored_games_list');
}

export async function restoreIgnoredGame(appId: number) {
	await invoke('restore_ignored_game', { appId });
}

export async function getTimelineSessions(limit = 200, offset = 0, gameId?: number) {
	return await invoke<TimelineSessionItem[]>('timeline_sessions', {
		limit,
		offset,
		game_id: gameId ?? null
	});
}

export async function syncSteamOwnedGameIcons(apiKey: string, steam64Id: string) {
	return await invoke<SteamIconSyncSummary>('sync_steam_owned_game_icons', {
		apiKey,
		steam64Id
	});
}

export async function getAnalyticsSnapshot(days = 30, topLimit = 8) {
	return await invoke<AnalyticsSnapshot>('analytics_snapshot', {
		days,
		top_limit: topLimit
	});
}
