<script lang="ts">
  // 导入侧边栏组件、CSS 样式文件、窗口控件
	import Sidebar from '$lib/components/Sidebar.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
	import './layout.css';
  import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/stores';
  import { loadSettingsFromStorage, type AppLanguage } from '$lib/settings';
  let { children } = $props();
  
	// 状态管理
  let darkMode = $state(false);
  let currentPage = $state('dashboard');
  let language = $state<AppLanguage>('zh-CN');

  // 主题切换函数
  function toggleTheme() {
    darkMode = !darkMode;

    document.documentElement.classList.toggle('dark', darkMode);
    localStorage.setItem('theme', darkMode ? 'dark' : 'light');
  }

  function navigateBySidebarPage(page: string) {
    let route: '/' | '/dashboard' | '/games' | '/timeline' | '/analytics' | '/settings' = '/dashboard';

    if (page === 'dashboard') {
      route = '/dashboard';
    } else if (page === 'games') {
      route = '/games';
    } else if (page === 'timeline') {
      route = '/timeline';
    } else if (page === 'analytics') {
      route = '/analytics';
    } else if (page === 'settings') {
      route = '/settings';
    }

    goto(resolve(route));
  }

  onMount(() => {
    const appSettings = loadSettingsFromStorage();
    language = appSettings.language;

    const saved = localStorage.getItem('theme');
    if (saved === 'dark') {
      darkMode = true;
    } else if (saved === 'light') {
      darkMode = false;
    } else {
      darkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }

    document.documentElement.classList.toggle('dark', darkMode);
    document.documentElement.lang = language;

    const handleThemeChange = (event: Event) => {
      const detail = event instanceof CustomEvent ? (event.detail as { darkMode?: boolean } | undefined) : undefined;
      if (detail && typeof detail.darkMode === 'boolean') {
        darkMode = detail.darkMode;
      }
    };

    window.addEventListener('pt-theme-change', handleThemeChange);

    const handleLanguageChange = (event: Event) => {
      const detail = event instanceof CustomEvent ? (event.detail as { language?: AppLanguage } | undefined) : undefined;
      if (detail && (detail.language === 'zh-CN' || detail.language === 'en-US')) {
        language = detail.language;
        document.documentElement.lang = language;
      }
    };

    window.addEventListener('pt-language-change', handleLanguageChange);

    return () => {
      window.removeEventListener('pt-theme-change', handleThemeChange);
      window.removeEventListener('pt-language-change', handleLanguageChange);
    };
  });

  $effect(() => {
    const path = $page.url.pathname;
    currentPage = path === '/' ? 'dashboard' : path.split('/')[1] ?? 'dashboard';
  });

</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="material-shell flex h-screen">
  <WindowControls />

  <!-- 侧边栏组件 -->
  <Sidebar 
    darkMode={darkMode}
    toggleTheme={toggleTheme}
    currentPage={currentPage}
    language={language}
    onNavigate={(page) => {
      navigateBySidebarPage(page);
    }}
  />
  
  <!-- 主内容区 -->
  <main class="material-main material-page flex-1 overflow-auto p-6 text-gray-900 transition-colors duration-200 dark:text-gray-100">
    {@render children()}
  </main>
</div>