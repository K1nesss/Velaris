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
	import AppIcon from '../../../icon.svg?raw';

	const navPages = ['dashboard', 'games', 'timeline', 'analytics', 'settings'];
	let visualPage = $state('dashboard');

	$effect(() => {
		visualPage = currentPage;
	});

	function getActiveIndex() {
		const activeIndex = navPages.indexOf(visualPage);
		return activeIndex === -1 ? 0 : activeIndex;
	}

	function navigateTo(pageName: string) {
		visualPage = pageName;
		onNavigate(pageName);
	}

	// 根据页面是否为当前活动页面返回不同的 CSS 类名，实现按钮的激活状态视觉效果。
	function getLinkClasses(pageName: string) {
		const isActive = visualPage === pageName;
		if (isActive) {
			return 'relative z-10 flex h-12 w-12 cursor-pointer items-center justify-center rounded-2xl text-blue-700 transition-colors duration-200 dark:text-blue-200';
		}
		return 'relative z-10 flex h-12 w-12 cursor-pointer items-center justify-center rounded-2xl text-slate-500 transition-colors duration-200 hover:text-slate-800 dark:text-slate-300 dark:hover:text-white';
	}

	function t(zh: string, en: string) {
		return language === 'zh-CN' ? zh : en;
	}
</script>

<aside
	class="sidebar-shell z-20 flex w-20 shrink-0 flex-col items-center border-r border-(--md-outline) bg-(--md-surface) py-6 transition-colors duration-200 md:flex"
>
	<nav class="flex w-full flex-1 flex-col items-center">
		<div
			class="sidebar-logo mb-6 flex h-14 w-14 items-center justify-center rounded-2xl border border-(--md-outline) bg-(--md-surface-2) p-2 shadow-sm ring-1 ring-slate-100 dark:ring-slate-800"
			aria-hidden="true"
		>
			<span class="app-icon-graphic">
				{@html AppIcon}
			</span>
		</div>

		<div class="relative flex w-full flex-col items-center gap-4">
			<div
				class="active-nav-indicator absolute top-0 left-1/2 h-12 w-12 rounded-2xl border border-blue-200 bg-blue-50 shadow-sm ring-1 ring-blue-100 dark:border-blue-800/60 dark:bg-blue-900/35 dark:ring-blue-900/50"
				style={`--active-offset: ${getActiveIndex() * 4}rem;`}
				aria-hidden="true"
			></div>

			<!-- Dashboard -->
			<button
				onclick={() => navigateTo('dashboard')}
				class={getLinkClasses('dashboard')}
				title={t('仪表盘', 'Dashboard')}
			>
				{@html DashboardIcon}
			</button>

			<!-- Games -->
			<button
				onclick={() => navigateTo('games')}
				class={getLinkClasses('games')}
				title={t('游戏库', 'Games Library')}
			>
				{@html GamesIcon}
			</button>

			<!-- Timeline -->
			<button
				onclick={() => navigateTo('timeline')}
				class={getLinkClasses('timeline')}
				title={t('时间线', 'Timeline')}
			>
				{@html TimelineIcon}
			</button>

			<!-- Analytics -->
			<button
				onclick={() => navigateTo('analytics')}
				class={getLinkClasses('analytics')}
				title={t('分析', 'Analytics')}
			>
				{@html AnalyticsIcon}
			</button>

			<!-- Settings -->
			<button
				onclick={() => navigateTo('settings')}
				class={getLinkClasses('settings')}
				title={t('设置', 'Settings')}
			>
				{@html SettingsIcon}
			</button>
		</div>
	</nav>

	<!-- Theme -->
	<div class="mt-auto">
		<button
			onclick={toggleTheme}
			class="sidebar-theme-button flex h-12 w-12 items-center justify-center rounded-2xl border border-(--md-outline) bg-(--md-surface-2) text-slate-600 transition hover:bg-slate-100 dark:text-slate-200 dark:hover:bg-slate-800"
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

<style>
	.sidebar-shell,
	.sidebar-logo,
	.sidebar-theme-button,
	.sidebar-shell :global(button),
	.sidebar-shell :global(svg) {
		transition: var(--theme-color-transition);
	}

	.app-icon-graphic {
		display: flex;
		height: 100%;
		width: 100%;
		align-items: center;
		justify-content: center;
	}

	.app-icon-graphic :global(svg) {
		display: block;
		height: 100%;
		width: 100%;
	}

	.active-nav-indicator {
		transform: translate3d(-50%, var(--active-offset), 0);
		will-change: transform;
		transition:
			transform 300ms cubic-bezier(0.16, 1, 0.3, 1),
			background-color var(--theme-duration) var(--theme-ease),
			border-color var(--theme-duration) var(--theme-ease),
			box-shadow var(--theme-duration) var(--theme-ease);
	}
</style>
