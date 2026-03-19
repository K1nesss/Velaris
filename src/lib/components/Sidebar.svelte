<script lang="ts">
  import type { AppLanguage } from '$lib/settings';

  // 定义了组件的属性接口，包含暗色模式状态、切换主题函数、当前页面和导航函数。
  interface Props {
    darkMode?: boolean;
    toggleTheme?: () => void;
    currentPage?: string;
    onNavigate?: (page: string) => void;
    language?: AppLanguage;
  }

  // 使用解构赋值从 $props() 获取传入的属性，并设置默认值。
  let {
    darkMode = false,
    toggleTheme = () => {},
    currentPage = 'dashboard',
    onNavigate = () => {},
    language = 'zh-CN'
  }: Props = $props();

  // 导入 SVG 作为组件
  import DashboardIcon from '$lib/assets/icons/dashboard.svg?raw';
  import GamesIcon from '$lib/assets/icons/games.svg?raw';
  import TimelineIcon from '$lib/assets/icons/timeline.svg?raw';
  import AnalyticsIcon from '$lib/assets/icons/analytics.svg?raw';
  import SettingsIcon from '$lib/assets/icons/settings.svg?raw';
  import SunIcon from '$lib/assets/icons/sun.svg?raw';
  import MoonIcon from '$lib/assets/icons/moon.svg?raw';

  // 根据页面是否为当前活动页面返回不同的 CSS 类名，实现按钮的激活状态视觉效果。
  function getLinkClasses(pageName: string) {
    const isActive = currentPage === pageName;
    if (isActive) {
      return "mb-4 flex h-12 w-12 cursor-pointer items-center justify-center rounded-2xl border border-blue-200 bg-blue-50 text-blue-700 shadow-sm ring-1 ring-blue-100 transition-all dark:border-blue-800/60 dark:bg-blue-900/35 dark:text-blue-200 dark:ring-blue-900/50";
    }
    return "mb-4 flex h-12 w-12 cursor-pointer items-center justify-center rounded-2xl text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-800 dark:text-slate-300 dark:hover:bg-slate-800/70 dark:hover:text-white";
  }

  function t(zh: string, en: string) {
    return language === 'zh-CN' ? zh : en;
  }
</script>

<aside class="z-20 flex w-20 shrink-0 flex-col items-center border-r border-(--md-outline) bg-(--md-surface) py-6 transition-colors duration-200 md:flex">
  <nav class="flex-1 w-full flex flex-col items-center">

    <!-- Dashboard -->
    <button
      onclick={() => onNavigate('dashboard')}
      class={getLinkClasses('dashboard')}
      title={t('仪表盘', 'Dashboard')}
    >
      {@html DashboardIcon}
    </button>

    <!-- Games -->
    <button
      onclick={() => onNavigate('games')}
      class={getLinkClasses('games')}
      title={t('游戏库', 'Games Library')}
    >
      {@html GamesIcon}
    </button>

    <!-- Timeline -->
    <button
      onclick={() => onNavigate('timeline')}
      class={getLinkClasses('timeline')}
      title={t('时间线', 'Timeline')}
    >
      {@html TimelineIcon}
    </button>

    <!-- Analytics -->
    <button
      onclick={() => onNavigate('analytics')}
      class={getLinkClasses('analytics')}
      title={t('分析', 'Analytics')}
    >
      {@html AnalyticsIcon}
    </button>

    <!-- Settings -->
    <button
      onclick={() => onNavigate('settings')}
      class={getLinkClasses('settings')}
      title={t('设置', 'Settings')}
    >
      {@html SettingsIcon}
    </button>
  </nav>

  <!-- Theme -->
  <div class="mt-auto">
    <button
      onclick={toggleTheme}
      class="flex h-12 w-12 items-center justify-center rounded-2xl border border-(--md-outline) bg-(--md-surface-2) text-slate-600 transition hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800"
      title={t('切换主题', 'Toggle Theme')}
    >
      {#if darkMode}
        {@html SunIcon}
      {:else}
        {@html MoonIcon}
      {/if}
    </button>
  </div>

</aside>
