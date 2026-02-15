<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  async function minimize() {
    await invoke('window_minimize');
  }

  async function close() {
    await invoke('window_close');
  }
</script>

<div
  class="fixed left-20 right-0 top-0 z-9999 flex h-6 items-stretch justify-end select-none cursor-default"
  data-tauri-drag-region
>
  <button
    on:click={minimize}
    data-tauri-drag-region="false"
    class="group h-6 w-10 text-gray-600 transition hover:bg-gray-200 active:bg-gray-300 dark:text-gray-200 dark:hover:bg-white/10 dark:active:bg-white/20"
    title="Minimize"
  >
    <span class="mx-auto block h-px w-3 bg-current opacity-80 group-hover:opacity-100"></span>
  </button>

  <button
    on:click={close}
    data-tauri-drag-region="false"
    class="group h-6 w-12 text-gray-600 transition hover:bg-red-500 hover:text-white active:bg-red-600 dark:text-gray-200"
    title="Close"
  >
    <span class="relative mx-auto block h-3 w-3">
      <span class="absolute left-1/2 top-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 rotate-45 bg-current opacity-80 group-hover:opacity-100"></span>
      <span class="absolute left-1/2 top-1/2 h-px w-3 -translate-x-1/2 -translate-y-1/2 -rotate-45 bg-current opacity-80 group-hover:opacity-100"></span>
    </span>
  </button>
</div>
