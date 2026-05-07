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
	import { checkForAppUpdate } from '$lib/updater';
	import {
		SETTINGS_STORAGE_KEY,
		loadSettingsFromStorage,
		mergeSettings,
		saveSettingsToStorage,
		type AppLanguage,
		type ThemeMode
	} from '$lib/settings';
	let { children } = $props();

	// 状态管理
	let darkMode = $state(false);
	let currentPage = $state('dashboard');
	let language = $state<AppLanguage>('zh-CN');
	let animationsEnabled = $state(true);
	let mainElement = $state<HTMLElement | null>(null);
	let activeScrollElement: HTMLElement | null = null;
	let showBackToTop = $state(false);
	let updateToast = $state({
		visible: false,
		version: ''
	});
	let hideBackToTopTimer: number | null = null;
	let hideUpdateToastTimer: number | null = null;
	const scrollPositions = new WeakMap<HTMLElement, number>();
	const GAMES_RESTORE_FROM_DETAIL_KEY = 'games_restore_from_detail_v1';
	const UPDATE_TOAST_STATE_KEY = 'velaris_update_toast_state_v1';

	function scheduleBackToTopHide() {
		if (hideBackToTopTimer !== null) {
			window.clearTimeout(hideBackToTopTimer);
		}

		hideBackToTopTimer = window.setTimeout(() => {
			showBackToTop = false;
			hideBackToTopTimer = null;
		}, 1800);
	}

	function handleMainScroll(event: Event) {
		if (!(event.target instanceof HTMLElement)) {
			return;
		}

		const target = event.target;
		const nextScrollTop = target.scrollTop;
		const previousScrollTop = scrollPositions.get(target) ?? nextScrollTop;
		const isScrollingUp = nextScrollTop < previousScrollTop;

		activeScrollElement = target;
		showBackToTop = isScrollingUp && nextScrollTop > 80;
		scrollPositions.set(target, nextScrollTop);

		if (showBackToTop) {
			scheduleBackToTopHide();
		}
	}

	function scrollMainToTop() {
		const target = activeScrollElement ?? mainElement;

		target?.scrollTo({
			top: 0,
			behavior: animationsEnabled ? 'smooth' : 'auto'
		});
		showBackToTop = false;
	}

	function applyAnimationPreference(enabled: boolean) {
		animationsEnabled = enabled;
		document.documentElement.classList.toggle('animations-disabled', !enabled);
	}

	function todayKey() {
		return new Date().toISOString().slice(0, 10);
	}

	function hasShownUpdateToastToday(version: string) {
		try {
			const raw = localStorage.getItem(UPDATE_TOAST_STATE_KEY);
			if (!raw) {
				return false;
			}
			const parsed = JSON.parse(raw) as { date?: string; version?: string };
			return parsed.date === todayKey() && parsed.version === version;
		} catch {
			return false;
		}
	}

	function markUpdateToastShown(version: string) {
		try {
			localStorage.setItem(
				UPDATE_TOAST_STATE_KEY,
				JSON.stringify({
					date: todayKey(),
					version
				})
			);
		} catch {
			// Ignore update reminder storage failures.
		}
	}

	function showUpdateToast(version: string) {
		updateToast = {
			visible: true,
			version
		};
		markUpdateToastShown(version);

		if (hideUpdateToastTimer !== null) {
			window.clearTimeout(hideUpdateToastTimer);
		}
		hideUpdateToastTimer = window.setTimeout(() => {
			updateToast.visible = false;
			hideUpdateToastTimer = null;
		}, 6500);
	}

	async function checkUpdateReminder() {
		try {
			const update = await checkForAppUpdate();
			if (!update || hasShownUpdateToastToday(update.version)) {
				return;
			}
			showUpdateToast(update.version);
			await update.close();
		} catch {
			// Automatic update checks should stay silent.
		}
	}

	function goToSettingsForUpdate() {
		updateToast.visible = false;
		void goto(resolve('/settings'));
	}

	// 主题切换函数
	function isDarkByTheme(themeMode: ThemeMode) {
		if (themeMode === 'dark') {
			return true;
		}
		if (themeMode === 'light') {
			return false;
		}
		return window.matchMedia('(prefers-color-scheme: dark)').matches;
	}

	function applyResolvedTheme(shouldUseDark: boolean) {
		darkMode = shouldUseDark;
		document.documentElement.classList.toggle('dark', darkMode);
		localStorage.setItem('theme', darkMode ? 'dark' : 'light');
	}

	function saveThemeMode(themeMode: ThemeMode) {
		const nextSettings = mergeSettings({
			...loadSettingsFromStorage(),
			themeMode
		});

		saveSettingsToStorage(nextSettings);
	}

	function toggleTheme() {
		const nextThemeMode: ThemeMode = darkMode ? 'light' : 'dark';

		saveThemeMode(nextThemeMode);
		applyResolvedTheme(nextThemeMode === 'dark');
		window.dispatchEvent(
			new CustomEvent('pt-theme-change', {
				detail: { darkMode }
			})
		);
	}

	function navigateBySidebarPage(page: string) {
		let route: '/' | '/dashboard' | '/games' | '/timeline' | '/analytics' | '/settings' =
			'/dashboard';

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

		try {
			sessionStorage.removeItem(GAMES_RESTORE_FROM_DETAIL_KEY);
		} catch {
			// Ignore navigation state cleanup failures.
		}

		void goto(resolve(route)).then(() => {
			window.requestAnimationFrame(() => {
				mainElement?.scrollTo({ top: 0 });
			});
		});
	}

	onMount(() => {
		const appSettings = loadSettingsFromStorage();
		language = appSettings.language;
		applyAnimationPreference(appSettings.animationsEnabled);

		let themeMode = appSettings.themeMode;
		const hasStoredSettings = localStorage.getItem(SETTINGS_STORAGE_KEY) !== null;
		const legacyTheme = localStorage.getItem('theme');
		if (!hasStoredSettings && (legacyTheme === 'dark' || legacyTheme === 'light')) {
			themeMode = legacyTheme;
			saveThemeMode(themeMode);
		}

		applyResolvedTheme(isDarkByTheme(themeMode));
		document.documentElement.lang = language;
		window.setTimeout(() => {
			void checkUpdateReminder();
		}, 3500);

		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		const handleSystemThemeChange = () => {
			if (loadSettingsFromStorage().themeMode === 'system') {
				applyResolvedTheme(mediaQuery.matches);
			}
		};

		const handleThemeChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { darkMode?: boolean } | undefined)
					: undefined;
			if (detail && typeof detail.darkMode === 'boolean') {
				darkMode = detail.darkMode;
			}
		};

		mediaQuery.addEventListener('change', handleSystemThemeChange);
		window.addEventListener('pt-theme-change', handleThemeChange);

		const handleLanguageChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { language?: AppLanguage } | undefined)
					: undefined;
			if (detail && (detail.language === 'zh-CN' || detail.language === 'en-US')) {
				language = detail.language;
				document.documentElement.lang = language;
			}
		};

		window.addEventListener('pt-language-change', handleLanguageChange);

		const preventNativeContextMenu = (event: MouseEvent) => {
			event.preventDefault();
		};

		window.addEventListener('contextmenu', preventNativeContextMenu);

		const handleAnimationChange = (event: Event) => {
			const detail =
				event instanceof CustomEvent
					? (event.detail as { animationsEnabled?: boolean } | undefined)
					: undefined;
			if (detail && typeof detail.animationsEnabled === 'boolean') {
				applyAnimationPreference(detail.animationsEnabled);
			}
		};

		window.addEventListener('pt-animations-change', handleAnimationChange);

		return () => {
			if (hideBackToTopTimer !== null) {
				window.clearTimeout(hideBackToTopTimer);
			}
			if (hideUpdateToastTimer !== null) {
				window.clearTimeout(hideUpdateToastTimer);
			}
			mediaQuery.removeEventListener('change', handleSystemThemeChange);
			window.removeEventListener('pt-theme-change', handleThemeChange);
			window.removeEventListener('pt-language-change', handleLanguageChange);
			window.removeEventListener('contextmenu', preventNativeContextMenu);
			window.removeEventListener('pt-animations-change', handleAnimationChange);
		};
	});

	$effect(() => {
		const path = $page.url.pathname;
		currentPage = path === '/' ? 'dashboard' : (path.split('/')[1] ?? 'dashboard');
		activeScrollElement = null;
		showBackToTop = false;
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="material-shell flex h-screen">
	<WindowControls />

	<!-- 侧边栏组件 -->
	<Sidebar
		{darkMode}
		{toggleTheme}
		{currentPage}
		{language}
		onNavigate={(page) => {
			navigateBySidebarPage(page);
		}}
	/>

	<!-- 主内容区 -->
	<main
		bind:this={mainElement}
		onscrollcapture={handleMainScroll}
		class="material-main material-page flex-1 overflow-auto text-gray-900 transition-colors duration-200 dark:text-gray-100"
	>
		{@render children()}
	</main>

	<button
		type="button"
		class={`back-to-top-button fixed right-6 bottom-6 z-40 flex h-11 w-11 items-center justify-center rounded-2xl border border-(--md-outline) bg-(--md-surface) text-(--md-text-muted) shadow-lg transition-all duration-200 hover:text-(--md-text) ${
			showBackToTop
				? 'pointer-events-auto translate-y-0 opacity-100'
				: 'pointer-events-none translate-y-3 opacity-0'
		}`}
		title={language === 'zh-CN' ? '返回顶部' : 'Back to top'}
		aria-label={language === 'zh-CN' ? '返回顶部' : 'Back to top'}
		onclick={scrollMainToTop}
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
			class="h-5 w-5"
			aria-hidden="true"
		>
			<path d="m18 15-6-6-6 6" />
		</svg>
	</button>

	{#if updateToast.visible}
		<div
			class="fixed right-6 bottom-20 z-50 w-[min(360px,calc(100vw-48px))] rounded-xl border border-(--md-outline) bg-(--md-surface) p-4 text-(--md-text) shadow-xl"
			role="status"
			aria-live="polite"
		>
			<div class="flex items-start justify-between gap-3">
				<div class="min-w-0">
					<p class="text-sm font-semibold">
						{language === 'zh-CN'
							? `发现新版本 ${updateToast.version}`
							: `New version ${updateToast.version}`}
					</p>
					<p class="mt-1 text-xs text-(--md-text-muted)">
						{language === 'zh-CN'
							? '可前往设置页查看更新内容'
							: 'Open Settings to review the update'}
					</p>
				</div>
				<button
					type="button"
					class="shrink-0 rounded-lg border border-(--md-outline) px-2.5 py-1.5 text-xs font-medium text-(--md-primary) transition hover:bg-(--md-surface-2)"
					onclick={goToSettingsForUpdate}
				>
					{language === 'zh-CN' ? '去设置' : 'Settings'}
				</button>
			</div>
		</div>
	{/if}
</div>
