export type ThemeMode = 'light' | 'dark' | 'system';
export type AppLanguage = 'zh-CN' | 'en-US';

export type AppSettings = {
  themeMode: ThemeMode;
  language: AppLanguage;
  apiKey: string;
  steam64Id: string;
  launchOnStartup: boolean;
  minimizeToTray: boolean;
  autoStartMonitor: boolean;
  dashboardRefreshSeconds: number;
  timelinePageSize: number;
  analyticsDefaultRange: number;
  animationsEnabled: boolean;
};

export const SETTINGS_STORAGE_KEY = 'pt-settings-v1';

// Keep defaults aligned with original pages behavior.
export const DEFAULT_SETTINGS: AppSettings = {
  themeMode: 'system',
  language: 'zh-CN',
  apiKey: '',
  steam64Id: '',
  launchOnStartup: false,
  minimizeToTray: true,
  autoStartMonitor: true,
  dashboardRefreshSeconds: 5,
  timelinePageSize: 80,
  analyticsDefaultRange: 7,
  animationsEnabled: true,
};

function normalizeRange(value: number, allow: number[], fallback: number) {
  return allow.includes(value) ? value : fallback;
}

export function mergeSettings(input: Partial<AppSettings>): AppSettings {
  const merged: AppSettings = {
    ...DEFAULT_SETTINGS,
    ...input,
  };

  merged.dashboardRefreshSeconds = normalizeRange(
    Number(merged.dashboardRefreshSeconds),
    [2, 5, 10, 30],
    DEFAULT_SETTINGS.dashboardRefreshSeconds,
  );
  merged.timelinePageSize = normalizeRange(
    Number(merged.timelinePageSize),
    [50, 80, 120, 200],
    DEFAULT_SETTINGS.timelinePageSize,
  );
  merged.analyticsDefaultRange = normalizeRange(
    Number(merged.analyticsDefaultRange),
    [1, 7, 30, 365],
    DEFAULT_SETTINGS.analyticsDefaultRange,
  );

  merged.apiKey = typeof merged.apiKey === 'string' ? merged.apiKey.trim() : '';
  merged.steam64Id =
    typeof merged.steam64Id === 'string' ? merged.steam64Id.trim() : '';

  if (!['light', 'dark', 'system'].includes(merged.themeMode)) {
    merged.themeMode = DEFAULT_SETTINGS.themeMode;
  }

  if (!['zh-CN', 'en-US'].includes(merged.language)) {
    merged.language = DEFAULT_SETTINGS.language;
  }

  return merged;
}

export function loadSettingsFromStorage(): AppSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) {
      return { ...DEFAULT_SETTINGS };
    }

    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return mergeSettings(parsed);
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

export function saveSettingsToStorage(settings: AppSettings) {
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
}
