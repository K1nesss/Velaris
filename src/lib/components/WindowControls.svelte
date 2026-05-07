<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	let suppressHover = $state(false);
	let hoverFallbackTimer: number | null = null;
	let pendingMouseRelease = false;

	function releaseHoverSuppression() {
		suppressHover = false;
		pendingMouseRelease = false;

		if (hoverFallbackTimer !== null) {
			window.clearTimeout(hoverFallbackTimer);
			hoverFallbackTimer = null;
		}

		window.removeEventListener('mousemove', releaseHoverSuppression);
	}

	function clearHoverState() {
		suppressHover = true;
		pendingMouseRelease = true;

		const active = document.activeElement;
		if (active instanceof HTMLElement) {
			active.blur();
		}

		window.removeEventListener('mousemove', releaseHoverSuppression);
		window.addEventListener('mousemove', releaseHoverSuppression, { once: true });

		if (hoverFallbackTimer !== null) {
			window.clearTimeout(hoverFallbackTimer);
		}

		// Fallback: release suppression even if no mouse move occurs.
		hoverFallbackTimer = window.setTimeout(() => {
			if (pendingMouseRelease) {
				releaseHoverSuppression();
			}
		}, 800);
	}

	async function minimize() {
		await invoke('window_minimize');
	}

	async function toggleMaximize() {
		await invoke('window_toggle_maximize');
	}

	async function close() {
		await invoke('window_close');
	}

	onMount(() => {
		const handleFocus = () => {
			clearHoverState();
		};

		const handleVisibilityChange = () => {
			if (document.visibilityState === 'visible') {
				clearHoverState();
			}
		};

		window.addEventListener('focus', handleFocus);
		document.addEventListener('visibilitychange', handleVisibilityChange);

		return () => {
			if (hoverFallbackTimer !== null) {
				window.clearTimeout(hoverFallbackTimer);
			}
			window.removeEventListener('mousemove', releaseHoverSuppression);
			window.removeEventListener('focus', handleFocus);
			document.removeEventListener('visibilitychange', handleVisibilityChange);
		};
	});
</script>

<div
	class={`window-controls fixed top-0 right-0 left-20 z-9999 flex h-7 cursor-default items-stretch justify-end select-none ${suppressHover ? 'pointer-events-none' : ''}`}
	data-tauri-drag-region
>
	<button
		onclick={minimize}
		data-tauri-drag-region="false"
		class={`group h-6 w-10 text-slate-600 transition active:bg-slate-300 dark:text-slate-200 dark:active:bg-slate-700 ${suppressHover ? '' : 'hover:bg-slate-200 dark:hover:bg-slate-700/60'}`}
		title="Minimize"
	>
		<span class="mx-auto block h-px w-3 bg-current opacity-80 group-hover:opacity-100"></span>
	</button>

	<button
		onclick={toggleMaximize}
		data-tauri-drag-region="false"
		class={`group h-6 w-10 text-slate-600 transition active:bg-slate-300 dark:text-slate-200 dark:active:bg-slate-700 ${suppressHover ? '' : 'hover:bg-slate-200 dark:hover:bg-slate-700/60'}`}
		title="Maximize"
	>
		<span class="relative mx-auto block h-3 w-3">
			<span class="absolute inset-0 border border-current opacity-80 group-hover:opacity-100"
			></span>
		</span>
	</button>

	<button
		onclick={close}
		data-tauri-drag-region="false"
		class={`group h-6 w-12 text-slate-600 transition active:bg-red-600 dark:text-slate-200 ${suppressHover ? '' : 'hover:bg-red-500 hover:text-white'}`}
		title="Close"
	>
		<span class="relative mx-auto block h-3 w-3">
			<span
				class="absolute top-1/2 left-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 bg-current opacity-80 group-hover:opacity-100"
			></span>
			<span
				class="absolute top-1/2 left-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 bg-current opacity-80 group-hover:opacity-100"
			></span>
		</span>
	</button>
</div>

<style>
	.window-controls :global(button),
	.window-controls :global(span) {
		transition: var(--theme-color-transition);
	}
</style>
