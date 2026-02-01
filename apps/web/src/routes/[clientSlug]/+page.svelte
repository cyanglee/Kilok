<script lang="ts">
	import type { PageData } from './$types';
	import { formatHours, monthNames } from '$lib/utils/formatters';
	import { StatsCard, QuotaProgressBar, MonthlyChart, EmptyState } from '$lib/components';

	let { data }: { data: PageData } = $props();

	// Prepare chart data with sessions info
	const chartData = data.monthlyStats.map((stat, i) => ({
		month: i + 1,
		hours: stat.hours,
		sessions: stat.sessions
	}));
</script>

<!-- Breadcrumb -->
<nav class="mb-6">
	<a href="/" class="text-text-muted hover:text-primary">
		<span class="inline-flex items-center gap-1">
			<svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M15 18l-6-6 6-6" />
			</svg>
			返回總覽
		</span>
	</a>
</nav>

<!-- Page Header -->
<div class="mb-8">
	<h1 class="text-3xl font-bold text-text">{data.client.name}</h1>
	<p class="mt-1 text-text-muted">
		{data.currentYear} 年度工時報告 · {data.projects.length} 個專案
	</p>
</div>

<!-- Bento Grid: Stats Cards -->
{#if data.monthlyHours > 0}
	<!-- 月額度模式：精簡顯示 -->
	<div class="mb-8 flex flex-wrap gap-4">
		<StatsCard
			label="累積可用額度"
			value={formatHours(data.availableQuota)}
			subtitle="含未用完結轉"
			valueClass={data.availableQuota < data.monthlyHours ? 'text-warning' : 'text-success'}
			highlighted={true}
		/>
		{#if data.carriedOver > 0}
			<StatsCard
				label="年度結轉"
				value="+{formatHours(data.carriedOver)}"
				valueClass="text-cyan-600"
			/>
		{/if}
	</div>
{:else}
	<!-- 年度額度模式：原有顯示 -->
	<div class="mb-8 grid grid-cols-2 gap-4 md:grid-cols-4">
		<StatsCard
			label="本月工時"
			value={formatHours(data.monthlyStats[new Date().getMonth()]?.hours ?? 0)}
			valueClass="text-primary"
		/>
		<StatsCard
			label="年度累計"
			value={formatHours(data.yearlyHours)}
			valueClass="text-text"
		/>
		<StatsCard
			label="合約額度"
			value={data.contractHours > 0 ? formatHours(data.contractHours) : '-'}
			valueClass="text-secondary"
		/>
		<StatsCard
			label="剩餘額度"
			value={data.contractHours > 0 ? formatHours(data.remainingHours) : '-'}
			valueClass={data.usagePercent > 90 ? 'text-danger' : data.usagePercent > 70 ? 'text-warning' : 'text-success'}
		/>
	</div>
{/if}

<!-- Usage Progress -->
{#if data.monthlyHours > 0 && data.yearlyQuota > 0}
	<div class="mb-8">
		<QuotaProgressBar
			label="年度額度使用進度"
			current={data.yearlyHours}
			quota={data.yearlyQuota}
			leftText="{formatHours(data.yearlyHours)} / {formatHours(data.yearlyQuota)} (12月 × {formatHours(data.monthlyHours)}{data.carriedOver > 0 ? ` + ${formatHours(data.carriedOver)} 結轉` : ''})"
			rightText="{data.yearlySessions} 個工作階段"
		/>
	</div>
{:else if data.contractHours > 0}
	<div class="mb-8">
		<QuotaProgressBar
			label="年度額度使用進度"
			current={data.yearlyHours}
			quota={data.contractHours}
			rightText="{data.yearlySessions} 個工作階段"
		/>
	</div>
{/if}

<!-- Monthly Chart -->
<div class="mb-8">
	<MonthlyChart
		data={chartData}
		currentMonth={new Date().getMonth() + 1}
		showSessions={true}
	/>
</div>

<!-- Monthly Reports List -->
<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-6 text-lg font-semibold text-text">月份報告</h2>

	{#if data.monthsWithData.length === 0}
		<EmptyState message="本年度尚無工作紀錄" />
	{:else}
		<div class="grid grid-cols-2 gap-3 md:grid-cols-4 lg:grid-cols-6">
			{#each data.monthsWithData.reverse() as stat}
				{@const period = `${data.currentYear}-${String(stat.month).padStart(2, '0')}`}
				<a
					href="/{data.client.slug}/{period}"
					class="group cursor-pointer rounded-lg border border-border p-4 text-center transition-all hover:border-primary hover:shadow-md"
				>
					<div class="text-sm font-medium text-text group-hover:text-primary">
						{monthNames[stat.month - 1]}
					</div>
					<div class="mt-1 text-lg font-bold text-text">{formatHours(stat.hours)}</div>
					<div class="text-xs text-text-muted">{stat.sessions} 階段</div>
				</a>
			{/each}
		</div>
	{/if}
</div>

<!-- Projects List -->
<div class="mt-8 rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-4 text-lg font-semibold text-text">相關專案</h2>
	<div class="space-y-2">
		{#each data.projects as project}
			<div class="flex items-center justify-between rounded-lg bg-background p-3">
				<div>
					<div class="font-medium text-text">{project.display_name ?? project.path}</div>
					<div class="font-mono text-xs text-text-muted">{project.path}</div>
				</div>
				{#if project.work_item_pattern}
					<code class="rounded bg-border px-2 py-1 text-xs text-text-muted">
						{project.work_item_pattern}
					</code>
				{/if}
			</div>
		{/each}
	</div>
</div>
