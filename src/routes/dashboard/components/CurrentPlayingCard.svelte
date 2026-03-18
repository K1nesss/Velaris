<script lang="ts">
  import type { CurrentPlayingGame } from '$lib/api';

  let { currentPlaying = null, loading = false } = $props<{
    currentPlaying?: CurrentPlayingGame | null;
    loading?: boolean;
  }>();

  function formatDateTime(timestamp: number) {
    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(new Date(timestamp * 1000));
  }
</script>

<article class="rounded-2xl border border-amber-200/60 bg-white/85 p-5 shadow-sm backdrop-blur-sm dark:border-amber-900/60 dark:bg-[#151926] lg:flex lg:h-48 lg:min-h-48 lg:flex-col">
  <div class="flex items-center justify-between gap-3">
    <div>
      <p class="text-xs font-medium uppercase tracking-[0.24em] text-amber-600 dark:text-amber-400">正在游玩</p>
      <!-- <h3 class="mt-2 text-lg font-semibold text-gray-900 dark:text-gray-100">当前进行中的会话</h3> -->
    </div>
  </div>

  {#if loading}
    <div class="mt-5 space-y-4 lg:flex lg:min-h-0 lg:flex-1 lg:flex-col">
      <div class="space-y-2">
        <div class="h-7 w-40 rounded-md skeleton-shimmer"></div>
        <div class="h-4 w-24 rounded-md skeleton-shimmer"></div>
      </div>

      <div class="grid gap-3 sm:grid-cols-2 lg:min-h-0 lg:flex-1 lg:content-end">
        <div class="rounded-xl bg-amber-50 p-4 dark:bg-amber-950/20">
          <div class="h-3 w-16 rounded-md skeleton-shimmer"></div>
          <div class="mt-2 h-6 w-24 rounded-md skeleton-shimmer"></div>
        </div>
        <div class="rounded-xl bg-gray-50 p-4 dark:bg-[#101522]">
          <div class="h-3 w-16 rounded-md skeleton-shimmer"></div>
          <div class="mt-2 h-5 w-32 rounded-md skeleton-shimmer"></div>
        </div>
      </div>
    </div>
  {:else if currentPlaying}
    <div class="mt-5 space-y-4 lg:flex lg:min-h-0 lg:flex-1 lg:flex-col">
      <div>
        <p class="text-2xl font-semibold text-gray-900 dark:text-gray-100">{currentPlaying.game_name}</p>
        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">会话 #{currentPlaying.session_id}</p>
      </div>

      <div class="grid gap-3 sm:grid-cols-2 lg:min-h-0 lg:flex-1 lg:content-end">
        <div class="rounded-xl bg-amber-50 p-4 dark:bg-amber-950/20">
          <p class="text-xs uppercase tracking-wide text-amber-700 dark:text-amber-300">已游玩</p>
          <p class="mt-2 text-xl font-semibold text-amber-900 dark:text-amber-100">{currentPlaying.formatted}</p>
        </div>
        <div class="rounded-xl bg-gray-50 p-4 dark:bg-[#101522]">
          <p class="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">开始时间</p>
          <p class="mt-2 text-base font-medium text-gray-900 dark:text-gray-100">{formatDateTime(currentPlaying.start_time)}</p>
        </div>
      </div>
    </div>
  {:else}
    <div class="mt-5 rounded-xl border border-dashed border-gray-200 bg-gray-50 px-4 py-8 text-center dark:border-gray-800 dark:bg-[#101522] lg:flex lg:flex-1 lg:items-center lg:justify-center">
      <p class="text-sm text-gray-500 dark:text-gray-400">当前没有正在进行中的游戏会话</p>
    </div>
  {/if}
</article>