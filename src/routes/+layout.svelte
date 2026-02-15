<script>
  // 导入侧边栏组件、CSS 样式文件、窗口控件
	import Sidebar from '$lib/components/Sidebar.svelte';
  import WindowControls from '$lib/components/WindowControls.svelte';
	import './layout.css';
  import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  let { children } = $props();
  
	// 状态管理
  let darkMode = $state(false);
  let currentPage = $state('dashboard');

  // 主题切换函数
  function toggleTheme() {
    darkMode = !darkMode;

    document.documentElement.classList.toggle('dark', darkMode);
    localStorage.setItem('theme', darkMode ? 'dark' : 'light');
  }

  onMount(() => {
    const saved = localStorage.getItem('theme');
    if (saved === 'dark') {
      darkMode = true;
    } else if (saved === 'light') {
      darkMode = false;
    } else {
      darkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }

    document.documentElement.classList.toggle('dark', darkMode);
  });

  $effect(() => {
    const path = $page.url.pathname;
    currentPage = path === '/' ? 'dashboard' : path.split('/')[1] ?? 'dashboard';
  });

</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex h-screen">
  <WindowControls />

  <!-- 侧边栏组件 -->
  <Sidebar 
    darkMode={darkMode}
    toggleTheme={toggleTheme}
    currentPage={currentPage}
    onNavigate={(page) => {
      goto(`/${page}`);
    }}
  />
  
  <!-- 主内容区 -->
  <main class="flex-1 overflow-auto bg-gray-100 dark:bg-[#0f1117] text-gray-900 dark:text-gray-100 p-6 transition-colors duration-200">
    {@render children()}
  </main>
</div>