<script lang="ts">
  import { onMount } from 'svelte';
  import { exportDatabase, importDatabase } from '$lib/api';
  import {
    type AppSettings,
    type AppLanguage,
    type ThemeMode,
    DEFAULT_SETTINGS,
    mergeSettings,
    loadSettingsFromStorage,
    saveSettingsToStorage,
  } from '$lib/settings';

  let settings = $state<AppSettings>({ ...DEFAULT_SETTINGS });
  let initialSnapshot = '';
  let savedMessage = $state('');
  let saving = $state(false);
  let maintenanceBusy = $state(false);
  let maintenanceResult = $state('');
  let importInput = $state<HTMLInputElement | null>(null);

  function serializeSettings(value: AppSettings) {
    return JSON.stringify(value);
  }

  function t(zh: string, en: string) {
    return settings.language === 'zh-CN' ? zh : en;
  }

  function tByLanguage(language: AppLanguage, zh: string, en: string) {
    return language === 'zh-CN' ? zh : en;
  }

  function isDarkByTheme(themeMode: ThemeMode) {
    if (themeMode === 'dark') {
      return true;
    }
    if (themeMode === 'light') {
      return false;
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }

  function applyThemePreference(themeMode: ThemeMode) {
    const shouldUseDark = isDarkByTheme(themeMode);

    document.documentElement.classList.toggle('dark', shouldUseDark);
    localStorage.setItem('theme', shouldUseDark ? 'dark' : 'light');
    window.dispatchEvent(
      new CustomEvent('pt-theme-change', {
        detail: { darkMode: shouldUseDark },
      }),
    );
  }

  function applyLanguagePreference(language: AppLanguage) {
    document.documentElement.lang = language;
    window.dispatchEvent(
      new CustomEvent('pt-language-change', {
        detail: { language },
      }),
    );
  }

  function loadSettings() {
    settings = loadSettingsFromStorage();
    initialSnapshot = serializeSettings(settings);
    applyThemePreference(settings.themeMode);
    applyLanguagePreference(settings.language);
  }

  function isDirty() {
    return serializeSettings(settings) !== initialSnapshot;
  }

  function resetToDefault() {
    settings = { ...DEFAULT_SETTINGS };
    applyThemePreference(settings.themeMode);
    applyLanguagePreference(settings.language);
    savedMessage = '';
  }

  async function saveSettings() {
    if (!isDirty()) {
      return;
    }

    saving = true;
    savedMessage = '';
    try {
      const merged = mergeSettings(settings);
      saveSettingsToStorage(merged);
      settings = merged;
      applyThemePreference(settings.themeMode);
      applyLanguagePreference(settings.language);
      initialSnapshot = serializeSettings(settings);
      savedMessage = tByLanguage(settings.language, '设置已保存', 'Settings saved');
      setTimeout(() => {
        savedMessage = '';
      }, 1800);
    } finally {
      saving = false;
    }
  }

  function base64ToBlob(base64: string, type = 'application/octet-stream') {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      bytes[index] = binary.charCodeAt(index);
    }
    return new Blob([bytes], { type });
  }

  function arrayBufferToBase64(buffer: ArrayBuffer) {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let index = 0; index < bytes.length; index += 1) {
      binary += String.fromCharCode(bytes[index]);
    }
    return btoa(binary);
  }

  async function runExportDatabase() {
    maintenanceBusy = true;
    maintenanceResult = '';

    try {
      const payload = await exportDatabase();
      const blob = base64ToBlob(payload, 'application/octet-stream');
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      const timestamp = new Date().toISOString().replace(/[.:]/g, '-');
      link.href = url;
      link.download = `playtime-tracker-${timestamp}.db`;
      link.click();
      URL.revokeObjectURL(url);
      maintenanceResult = t('数据库已导出', 'Database exported');
    } catch (error) {
      maintenanceResult = error instanceof Error ? error.message : String(error);
    } finally {
      maintenanceBusy = false;
    }
  }

  async function runImportDatabase(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) {
      return;
    }

    maintenanceBusy = true;
    maintenanceResult = '';

    try {
      const buffer = await file.arrayBuffer();
      const payload = arrayBufferToBase64(buffer);
      await importDatabase(payload);
      maintenanceResult = t(
        '数据库导入成功，请重启应用以确保监控服务使用新数据',
        'Database imported successfully. Please restart the app to ensure monitor service uses new data.',
      );
    } catch (error) {
      maintenanceResult = error instanceof Error ? error.message : String(error);
    } finally {
      maintenanceBusy = false;
      target.value = '';
    }
  }

  onMount(() => {
    loadSettings();

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemThemeChange = () => {
      if (settings.themeMode === 'system') {
        applyThemePreference('system');
      }
    };

    mediaQuery.addEventListener('change', handleSystemThemeChange);

    return () => {
      mediaQuery.removeEventListener('change', handleSystemThemeChange);
    };
  });

  $effect(() => {
    applyThemePreference(settings.themeMode);
    applyLanguagePreference(settings.language);
  });
</script>

<section class="space-y-6 p-4 md:p-8">
  <header class="flex flex-wrap items-start justify-between gap-4">
    <div>
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{t('设置', 'Settings')}</h1>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('管理主题、语言、数据展示和应用行为偏好', 'Manage theme, language, data display and app behavior preferences')}</p>
    </div>

    <div class="flex items-center gap-2">
      <button
        class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-60 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200 dark:hover:bg-gray-800"
        onclick={resetToDefault}
        disabled={saving}
      >
        {t('恢复默认', 'Reset Defaults')}
      </button>
      <button
        class="rounded-lg bg-blue-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-60"
        onclick={saveSettings}
        disabled={!isDirty() || saving}
      >
        {saving ? t('保存中...', 'Saving...') : t('保存设置', 'Save Settings')}
      </button>
    </div>
  </header>

  {#if savedMessage}
    <p class="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700 dark:border-emerald-900/50 dark:bg-emerald-950/20 dark:text-emerald-300">
      {savedMessage}
    </p>
  {/if}

  <div class="grid gap-6 xl:grid-cols-2">
    <article class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
      <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">{t('外观', 'Appearance')}</h2>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('主题、语言与动效显示偏好', 'Theme, language and animation preferences')}</p>

      <div class="mt-4 space-y-4">
        <div>
          <p class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">{t('语言', 'Language')}</p>
          <div class="inline-flex rounded-lg border border-gray-200 p-1 dark:border-gray-700">
            <button
              class={`rounded-md px-3 py-1.5 text-sm transition ${settings.language === 'zh-CN' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
              onclick={() => {
                settings.language = 'zh-CN';
              }}
            >
              中文
            </button>
            <button
              class={`rounded-md px-3 py-1.5 text-sm transition ${settings.language === 'en-US' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
              onclick={() => {
                settings.language = 'en-US';
              }}
            >
              English
            </button>
          </div>
        </div>

        <div>
          <p class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">{t('主题模式', 'Theme Mode')}</p>
          <div class="inline-flex rounded-lg border border-gray-200 p-1 dark:border-gray-700">
            <button
              class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'light' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
              onclick={() => {
                settings.themeMode = 'light';
              }}
            >
              {t('浅色', 'Light')}
            </button>
            <button
              class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'dark' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
              onclick={() => {
                settings.themeMode = 'dark';
              }}
            >
              {t('深色', 'Dark')}
            </button>
            <button
              class={`rounded-md px-3 py-1.5 text-sm transition ${settings.themeMode === 'system' ? 'bg-blue-600 text-white' : 'text-gray-600 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-800'}`}
              onclick={() => {
                settings.themeMode = 'system';
              }}
            >
              {t('跟随系统', 'System')}
            </button>
          </div>
        </div>

        <label class="flex items-center justify-between rounded-lg border border-gray-200 px-3 py-2 dark:border-gray-700">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('启用界面动效', 'Enable UI Animations')}</span>
          <input type="checkbox" bind:checked={settings.animationsEnabled} class="h-4 w-4" />
        </label>
      </div>
    </article>

    <article class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
      <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">{t('应用行为', 'App Behavior')}</h2>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('启动与后台运行策略', 'Startup and background-running strategy')}</p>

      <div class="mt-4 space-y-3">
        <label class="flex items-center justify-between rounded-lg border border-gray-200 px-3 py-2 dark:border-gray-700">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('开机自动启动', 'Launch on Startup')}</span>
          <input type="checkbox" bind:checked={settings.launchOnStartup} class="h-4 w-4" />
        </label>

        <label class="flex items-center justify-between rounded-lg border border-gray-200 px-3 py-2 dark:border-gray-700">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('关闭时最小化到托盘', 'Minimize to Tray on Close')}</span>
          <input type="checkbox" bind:checked={settings.minimizeToTray} class="h-4 w-4" />
        </label>

        <label class="flex items-center justify-between rounded-lg border border-gray-200 px-3 py-2 dark:border-gray-700">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('应用启动后自动监控进程', 'Auto Start Process Monitor')}</span>
          <input type="checkbox" bind:checked={settings.autoStartMonitor} class="h-4 w-4" />
        </label>
      </div>
    </article>

    <article class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926] xl:col-span-2">
      <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">{t('数据展示', 'Data Display')}</h2>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('控制 dashboard、timeline、analytics 默认展示参数', 'Control default display parameters for dashboard, timeline and analytics')}</p>

      <div class="mt-4 grid gap-4 md:grid-cols-3">
        <label class="space-y-2">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('Dashboard 刷新间隔', 'Dashboard Refresh Interval')}</span>
          <select
            bind:value={settings.dashboardRefreshSeconds}
            class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-800 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100"
          >
            <option value={2}>{settings.language === 'zh-CN' ? '2 秒' : '2 sec'}</option>
            <option value={5}>{settings.language === 'zh-CN' ? '5 秒' : '5 sec'}</option>
            <option value={10}>{settings.language === 'zh-CN' ? '10 秒' : '10 sec'}</option>
            <option value={30}>{settings.language === 'zh-CN' ? '30 秒' : '30 sec'}</option>
          </select>
        </label>

        <label class="space-y-2">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('Timeline 每次加载条数', 'Timeline Page Size')}</span>
          <select
            bind:value={settings.timelinePageSize}
            class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-800 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100"
          >
            <option value={50}>50</option>
            <option value={80}>80</option>
            <option value={120}>120</option>
            <option value={200}>200</option>
          </select>
        </label>

        <label class="space-y-2">
          <span class="text-sm text-gray-700 dark:text-gray-200">{t('Analytics 默认区间', 'Analytics Default Range')}</span>
          <select
            bind:value={settings.analyticsDefaultRange}
            class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3 text-sm text-gray-800 outline-none transition focus:border-blue-500 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-100"
          >
            <option value={1}>{t('一天', '1 day')}</option>
            <option value={7}>{t('一周', '7 days')}</option>
            <option value={30}>{t('一月', '30 days')}</option>
            <option value={365}>{t('一年', '1 year')}</option>
          </select>
        </label>
      </div>
    </article>

    <article class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926] xl:col-span-2">
      <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">{t('数据维护', 'Data Maintenance')}</h2>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">{t('导出或导入数据库文件', 'Export or import database file')}</p>

      <div class="mt-4 flex flex-wrap gap-3">
        <button
          class="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm font-medium text-gray-700 transition hover:bg-gray-50 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200 dark:hover:bg-gray-800"
          onclick={runExportDatabase}
          disabled={maintenanceBusy}
        >
          {maintenanceBusy ? t('处理中...', 'Processing...') : t('导出数据库', 'Export Database')}
        </button>

        <button
          class="rounded-lg border border-blue-300 bg-blue-50 px-3 py-2 text-sm font-medium text-blue-700 transition hover:bg-blue-100 dark:border-blue-900 dark:bg-blue-950/30 dark:text-blue-200 dark:hover:bg-blue-950/50"
          onclick={() => importInput?.click()}
          disabled={maintenanceBusy}
        >
          {t('导入数据库', 'Import Database')}
        </button>

        <input
          bind:this={importInput}
          type="file"
          class="hidden"
          accept=".db,.sqlite,.sqlite3"
          onchange={runImportDatabase}
        />
      </div>

      {#if maintenanceResult}
        <p class="mt-3 rounded-lg border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700 dark:border-gray-700 dark:bg-[#101522] dark:text-gray-200">
          {maintenanceResult}
        </p>
      {/if}
    </article>
  </div>
</section>
