<script lang="ts">
  // 定义了组件的属性接口，包含暗色模式状态、切换主题函数、当前页面和导航函数。
  interface Props {
    darkMode?: boolean;
    toggleTheme?: () => void;
    currentPage?: string;
    onNavigate?: (page: string) => void;
  }

  // 使用解构赋值从 $props() 获取传入的属性，并设置默认值。
  let {
    darkMode = false,
    toggleTheme = () => {},
    currentPage = 'dashboard',
    onNavigate = () => {}
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
      return "flex flex-col items-center justify-center w-12 h-12 rounded-xl bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-300 ring-1 ring-blue-200 dark:ring-blue-400/30 shadow-sm transition-all cursor-pointer mb-4";
    }
    return "flex flex-col items-center justify-center w-12 h-12 rounded-xl text-gray-500 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/12 hover:text-gray-900 dark:hover:text-white transition-colors cursor-pointer mb-4";
  }
</script>

<aside class="w-20 bg-white dark:bg-[#151926] border-r border-gray-200 dark:border-gray-800 flex flex-col items-center py-6 transition-colors duration-200 md:flex shrink-0 z-20">
  <nav class="flex-1 w-full flex flex-col items-center">

    <!-- Dashboard -->
    <button
      onclick={() => onNavigate('dashboard')}
      class={getLinkClasses('dashboard')}
      title="Dashboard"
    >
      {@html DashboardIcon}
    </button>

    <!-- Games -->
    <button
      onclick={() => onNavigate('games')}
      class={getLinkClasses('games')}
      title="Games Library"
    >
      {@html GamesIcon}
    </button>

    <!-- Timeline -->
    <button
      onclick={() => onNavigate('timeline')}
      class={getLinkClasses('timeline')}
      title="Timeline"
    >
      {@html TimelineIcon}
    </button>

    <!-- Analytics -->
    <button
      onclick={() => onNavigate('analytics')}
      class={getLinkClasses('analytics')}
      title="Analytics"
    >
      {@html AnalyticsIcon}
    </button>

    <!-- Settings -->
    <button
      onclick={() => onNavigate('settings')}
      class={getLinkClasses('settings')}
      title="Settings"
    >
      {@html SettingsIcon}
    </button>
  </nav>

  <!-- Theme -->
  <div class="mt-auto">
    <button
      onclick={toggleTheme}
      class="flex items-center justify-center w-12 h-12 rounded-xl bg-gray-100 dark:bg-gray-800/90 border border-gray-200 dark:border-gray-700 hover:bg-gray-200 dark:hover:bg-gray-700 transition text-gray-600 dark:text-gray-200"
      title="Toggle Theme"
    >
      {#if darkMode}
        {@html SunIcon}
      {:else}
        {@html MoonIcon}
      {/if}
    </button>
  </div>

</aside>
