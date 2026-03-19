<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  async function minimize() {
    await invoke('window_minimize');
  }

  async function toggleMaximize() {
    await invoke('window_toggle_maximize');
  }

  async function close() {
    await invoke('window_close');
  }
</script>

<div
  class="fixed left-20 right-0 top-0 z-9999 flex h-7 cursor-default items-stretch justify-end select-none"
  data-tauri-drag-region
>
  <button
    on:click={minimize}
    data-tauri-drag-region="false"
    class="group h-6 w-10 text-slate-600 transition hover:bg-slate-200 active:bg-slate-300 dark:text-slate-200 dark:hover:bg-slate-700/60 dark:active:bg-slate-700"
    title="Minimize"
  >
    <span class="mx-auto block h-px w-3 bg-current opacity-80 group-hover:opacity-100"></span>
  </button>

  <button
    on:click={toggleMaximize}
    data-tauri-drag-region="false"
    class="group h-6 w-10 text-slate-600 transition hover:bg-slate-200 active:bg-slate-300 dark:text-slate-200 dark:hover:bg-slate-700/60 dark:active:bg-slate-700"
    title="Maximize"
  >
    <span class="relative mx-auto block h-3 w-3">
      <span class="absolute inset-0 border border-current opacity-80 group-hover:opacity-100"></span>
    </span>
  </button>

  <button
    on:click={close}
    data-tauri-drag-region="false"
    class="group h-6 w-12 text-slate-600 transition hover:bg-red-500 hover:text-white active:bg-red-600 dark:text-slate-200"
    title="Close"
  >
    <span class="relative mx-auto block h-3 w-3">
      <span class="absolute left-1/2 top-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 bg-current opacity-80 group-hover:opacity-100"></span>
      <span class="absolute left-1/2 top-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 bg-current opacity-80 group-hover:opacity-100"></span>
    </span>
  </button>
</div>
