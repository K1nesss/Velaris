<script lang="ts">
  import {
    getDashboardCurrentPlaying,
    getDashboardDailyChart,
    getDashboardDonutData,
    getDashboardRecentSessions,
    getDashboardTodayTotal,
    getDashboardWeekTotal,
    hello,
    printDatabaseTables,
  } from '$lib/api';

  let testing = $state(false);
  let lastAction = $state('');
  let lastResult = $state('点击下面按钮后，这里会显示接口返回值。');
  let lastError = $state('');

  async function test() {
    const result = await hello();
    console.log(result);
  }

  async function printTables() {
    await printDatabaseTables();
  }

  async function runApiTest(label: string, request: () => Promise<unknown>) {
    testing = true;
    lastAction = label;
    lastError = '';

    try {
      const result = await request();
      lastResult = JSON.stringify(result, null, 2);
      console.log(`[settings test] ${label}`, result);
      return result;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      lastError = message;
      lastResult = '';
      console.error(`[settings test] ${label} failed`, error);
      return null;
    } finally {
      testing = false;
    }
  }

  async function testTodayTotal() {
    await runApiTest('dashboard_today_total', () => getDashboardTodayTotal());
  }

  async function testWeekTotal() {
    await runApiTest('dashboard_week_total', () => getDashboardWeekTotal());
  }

  async function testDailyChart() {
    await runApiTest('dashboard_daily_chart', () => getDashboardDailyChart(7));
  }

  async function testCurrentPlaying() {
    await runApiTest('dashboard_current_playing', () => getDashboardCurrentPlaying());
  }

  async function testDonutData() {
    await runApiTest('dashboard_donut_data', () => getDashboardDonutData(10));
  }

  async function testRecentSessions() {
    await runApiTest('dashboard_recent_sessions', () => getDashboardRecentSessions(10));
  }

  async function testAllDashboardApis() {
    await testTodayTotal();
    await testWeekTotal();
    await testDailyChart();
    await testCurrentPlaying();
    await testDonutData();
    await testRecentSessions();
  }
</script>

<section class="space-y-6">
  <article class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-800 dark:bg-[#151926]">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-xs font-medium uppercase tracking-[0.24em] text-gray-500 dark:text-gray-400">Settings</p>
        <h2 class="mt-2 text-2xl font-semibold text-gray-900 dark:text-gray-100">后端联调测试</h2>
      </div>
      <span class="rounded-full bg-gray-100 px-3 py-1 text-xs text-gray-700 dark:bg-gray-800 dark:text-gray-300">
        开发调试
      </span>
    </div>

    <div class="mt-5 flex flex-wrap gap-3">
      <button
        onclick={test}
        class="inline-flex items-center rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 dark:bg-blue-500 dark:hover:bg-blue-400"
      >
        Test Rust
      </button>
      <button
        onclick={printTables}
        class="inline-flex items-center rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 dark:border-gray-700 dark:bg-[#151926] dark:text-gray-200 dark:hover:bg-gray-800"
      >
        Print DB Tables
      </button>
      <button
        onclick={testTodayTotal}
        class="inline-flex items-center rounded-lg border border-cyan-200 bg-cyan-50 px-4 py-2 text-sm font-medium text-cyan-700 hover:bg-cyan-100 dark:border-cyan-900 dark:bg-cyan-950/40 dark:text-cyan-200 dark:hover:bg-cyan-950/70"
      >
        Test Today Total
      </button>
      <button
        onclick={testWeekTotal}
        class="inline-flex items-center rounded-lg border border-purple-200 bg-purple-50 px-4 py-2 text-sm font-medium text-purple-700 hover:bg-purple-100 dark:border-purple-900 dark:bg-purple-950/40 dark:text-purple-200 dark:hover:bg-purple-950/70"
      >
        Test Week Total
      </button>
      <button
        onclick={testDailyChart}
        class="inline-flex items-center rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-2 text-sm font-medium text-emerald-700 hover:bg-emerald-100 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200 dark:hover:bg-emerald-950/70"
      >
        Test Daily Chart
      </button>
      <button
        onclick={testCurrentPlaying}
        class="inline-flex items-center rounded-lg border border-amber-200 bg-amber-50 px-4 py-2 text-sm font-medium text-amber-700 hover:bg-amber-100 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200 dark:hover:bg-amber-950/70"
      >
        Test Current Playing
      </button>
      <button
        onclick={testDonutData}
        class="inline-flex items-center rounded-lg border border-pink-200 bg-pink-50 px-4 py-2 text-sm font-medium text-pink-700 hover:bg-pink-100 dark:border-pink-900 dark:bg-pink-950/40 dark:text-pink-200 dark:hover:bg-pink-950/70"
      >
        Test Donut Data
      </button>
      <button
        onclick={testRecentSessions}
        class="inline-flex items-center rounded-lg border border-slate-200 bg-slate-50 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200 dark:hover:bg-slate-800"
      >
        Test Recent Sessions
      </button>
      <button
        onclick={testAllDashboardApis}
        class="inline-flex items-center rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-emerald-500 dark:hover:bg-emerald-400"
        disabled={testing}
      >
        {testing ? 'Testing...' : 'Test All Dashboard APIs'}
      </button>
    </div>

    <div class="mt-5 rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-gray-700 dark:bg-[#0f1320]">
      <div class="flex items-center justify-between gap-3">
        <p class="text-sm font-medium text-gray-900 dark:text-gray-100">最近一次测试结果</p>
        {#if lastAction}
          <span class="rounded-full bg-gray-200 px-2.5 py-1 text-xs text-gray-700 dark:bg-gray-800 dark:text-gray-300">
            {lastAction}
          </span>
        {/if}
      </div>

      {#if lastError}
        <p class="mt-3 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/30 dark:text-red-300">
          {lastError}
        </p>
      {/if}

      <pre class="mt-3 overflow-x-auto rounded-lg bg-white p-3 text-xs leading-6 text-gray-800 dark:bg-[#111827] dark:text-gray-200">{lastResult}</pre>
    </div>
  </article>
</section>