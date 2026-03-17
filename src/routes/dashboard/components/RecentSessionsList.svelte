<script lang="ts">
  import type { RecentSessionItem } from '$lib/api';

  let { sessions = [], loading = false } = $props<{
    sessions?: RecentSessionItem[];
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

<article class="flex h-96 min-h-96 flex-col overflow-hidden rounded-2xl border border-gray-200 bg-white/90 p-5 shadow-sm backdrop-blur-sm dark:border-gray-800 dark:bg-[#151926]">
  <div class="flex items-center justify-between gap-3">
    <div>
      <p class="text-xs font-medium uppercase tracking-[0.24em] text-slate-500 dark:text-slate-400">最近会话</p>
      <!-- <h3 class="mt-2 text-lg font-semibold text-gray-900 dark:text-gray-100">最近游玩记录</h3> -->
    </div>
    {#if loading}
      <span class="rounded-full bg-slate-100 px-3 py-1 text-xs text-slate-700 dark:bg-slate-800 dark:text-slate-300">
        加载中
      </span>
    {/if}
  </div>

  {#if sessions.length > 0}
    <div class="mt-4 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
      {#each sessions as session (session.session_id)}
        <div class="rounded-xl border border-gray-100 bg-gray-50 p-3 dark:border-gray-800 dark:bg-[#101522]">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">{session.game_name}</p>
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">开始于 {formatDateTime(session.start_time)}</p>
            </div>
            <span class="shrink-0 rounded-full bg-slate-200 px-2.5 py-1 text-xs text-slate-700 dark:bg-slate-800 dark:text-slate-300">
              {session.formatted}
            </span>
          </div>

          <div class="mt-2 flex items-center justify-between gap-3 text-xs text-gray-500 dark:text-gray-400">
            <span>Game #{session.game_id}</span>
            <span>{session.end_time ? `结束于 ${formatDateTime(session.end_time)}` : '仍在进行中'}</span>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="mt-5 flex flex-1 items-center justify-center rounded-xl border border-dashed border-gray-200 bg-gray-50 px-4 py-8 text-center dark:border-gray-800 dark:bg-[#101522]">
      <p class="text-sm text-gray-500 dark:text-gray-400">还没有可展示的最近会话</p>
    </div>
  {/if}
</article>