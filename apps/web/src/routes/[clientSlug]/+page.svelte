<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	function formatHours(hours: number): string {
		const h = Math.floor(hours);
		const m = Math.round((hours - h) * 60);
		return m > 0 ? `${h}h ${m}m` : `${h}h`;
	}

	const monthNames = [
		'一月',
		'二月',
		'三月',
		'四月',
		'五月',
		'六月',
		'七月',
		'八月',
		'九月',
		'十月',
		'十一月',
		'十二月'
	];

	// Calculate max hours for chart scaling
	const maxHours = Math.max(...data.monthlyStats.map((m) => m.hours), 1);
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
	<!-- 月額度模式：顯示月度相關統計 -->
	<div class="mb-8 grid grid-cols-2 gap-4 md:grid-cols-4 lg:grid-cols-5">
		<!-- 當月額度 -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">當月額度</div>
			<div class="mt-1 text-2xl font-bold text-secondary">
				{formatHours(data.monthlyHours)}
			</div>
		</div>

		<!-- 當月已使用 -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">本月已使用</div>
			<div class="mt-1 text-2xl font-bold text-primary">
				{formatHours(data.currentMonthHours)}
			</div>
			<div class="mt-1 text-xs text-text-muted">
				{data.monthlyUsagePercent.toFixed(0)}% 月額度
			</div>
		</div>

		<!-- 累積可用額度 -->
		<div
			class="rounded-xl border-2 border-primary/30 bg-primary/5 p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-primary/70">累積可用額度</div>
			<div
				class="mt-1 text-2xl font-bold {data.availableQuota < data.monthlyHours
					? 'text-warning'
					: 'text-success'}"
			>
				{formatHours(data.availableQuota)}
			</div>
			<div class="mt-1 text-xs text-text-muted">
				含未用完結轉
			</div>
		</div>

		<!-- 年度累計 -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">年度累計</div>
			<div class="mt-1 text-2xl font-bold text-text">{formatHours(data.yearlyHours)}</div>
			<div class="mt-1 text-xs text-text-muted">
				/ {formatHours(data.accumulatedQuota)} 可用
			</div>
		</div>

		<!-- 結轉時數 -->
		{#if data.carriedOver > 0}
			<div
				class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
			>
				<div class="text-xs font-medium text-text-muted">年度結轉</div>
				<div class="mt-1 text-2xl font-bold text-cyan-600">
					+{formatHours(data.carriedOver)}
				</div>
			</div>
		{/if}
	</div>
{:else}
	<!-- 年度額度模式：原有顯示 -->
	<div class="mb-8 grid grid-cols-2 gap-4 md:grid-cols-4">
		<!-- This Month -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">本月工時</div>
			<div class="mt-1 text-2xl font-bold text-primary">
				{formatHours(data.monthlyStats[new Date().getMonth()]?.hours ?? 0)}
			</div>
		</div>

		<!-- Yearly Total -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">年度累計</div>
			<div class="mt-1 text-2xl font-bold text-text">{formatHours(data.yearlyHours)}</div>
		</div>

		<!-- Contract Hours -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">合約額度</div>
			<div class="mt-1 text-2xl font-bold text-secondary">
				{data.contractHours > 0 ? formatHours(data.contractHours) : '-'}
			</div>
		</div>

		<!-- Remaining -->
		<div
			class="rounded-xl border border-border bg-surface p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="text-xs font-medium text-text-muted">剩餘額度</div>
			<div
				class="mt-1 text-2xl font-bold {data.usagePercent > 90
					? 'text-danger'
					: data.usagePercent > 70
						? 'text-warning'
						: 'text-success'}"
			>
				{data.contractHours > 0 ? formatHours(data.remainingHours) : '-'}
			</div>
		</div>
	</div>
{/if}

<!-- Usage Progress -->
{#if data.monthlyHours > 0}
	<!-- 月額度模式進度條 -->
	<div class="mb-8 space-y-4">
		<!-- 當月額度進度 -->
		<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
			<div class="mb-2 flex justify-between">
				<span class="text-sm font-medium text-text">本月額度使用進度</span>
				<span
					class="text-sm font-bold {data.monthlyUsagePercent > 100
						? 'text-danger'
						: data.monthlyUsagePercent > 80
							? 'text-warning'
							: 'text-text'}"
				>
					{data.monthlyUsagePercent.toFixed(1)}%
				</span>
			</div>
			<div class="h-3 overflow-hidden rounded-full bg-border">
				<div
					class="h-full rounded-full transition-all {data.monthlyUsagePercent > 100
						? 'bg-danger'
						: data.monthlyUsagePercent > 80
							? 'bg-warning'
							: 'bg-primary'}"
					style="width: {Math.min(100, data.monthlyUsagePercent)}%"
				></div>
			</div>
			<div class="mt-2 flex justify-between text-xs text-text-muted">
				<span>{formatHours(data.currentMonthHours)} / {formatHours(data.monthlyHours)} 月額度</span>
				<span>
					{#if data.monthlyUsagePercent > 100}
						超額 {formatHours(data.currentMonthHours - data.monthlyHours)}
					{:else}
						剩餘 {formatHours(data.monthlyHours - data.currentMonthHours)}
					{/if}
				</span>
			</div>
		</div>

		<!-- 累積額度進度 -->
		{#if true}
			{@const accumulatedPercent = data.accumulatedQuota > 0 ? (data.yearlyHours / data.accumulatedQuota) * 100 : 0}
			<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
			<div class="mb-2 flex justify-between">
				<span class="text-sm font-medium text-text">累積額度使用進度</span>
				<span
					class="text-sm font-bold {accumulatedPercent > 90
						? 'text-danger'
						: accumulatedPercent > 70
							? 'text-warning'
							: 'text-text'}"
				>
					{accumulatedPercent.toFixed(1)}%
				</span>
			</div>
			<div class="h-3 overflow-hidden rounded-full bg-border">
				<div
					class="h-full rounded-full transition-all {accumulatedPercent > 90
						? 'bg-danger'
						: accumulatedPercent > 70
							? 'bg-warning'
							: 'bg-primary'}"
					style="width: {Math.min(100, accumulatedPercent)}%"
				></div>
			</div>
			<div class="mt-2 flex justify-between text-xs text-text-muted">
				<span>
					{formatHours(data.yearlyHours)} / {formatHours(data.accumulatedQuota)}
					<span class="text-text-muted/60">
						({data.currentMonth}月 × {formatHours(data.monthlyHours)}{#if data.carriedOver > 0} + {formatHours(data.carriedOver)} 結轉{/if})
					</span>
				</span>
				<span>{data.yearlySessions} 個工作階段</span>
			</div>
			</div>
		{/if}
	</div>
{:else if data.contractHours > 0}
	<!-- 年度額度模式進度條 -->
	<div class="mb-8 rounded-xl border border-border bg-surface p-6 shadow-sm">
		<div class="mb-2 flex justify-between">
			<span class="text-sm font-medium text-text">年度額度使用進度</span>
			<span
				class="text-sm font-bold {data.usagePercent > 90
					? 'text-danger'
					: data.usagePercent > 70
						? 'text-warning'
						: 'text-text'}"
			>
				{data.usagePercent.toFixed(1)}%
			</span>
		</div>
		<div class="h-3 overflow-hidden rounded-full bg-border">
			<div
				class="h-full rounded-full transition-all {data.usagePercent > 90
					? 'bg-danger'
					: data.usagePercent > 70
						? 'bg-warning'
						: 'bg-primary'}"
				style="width: {Math.min(100, data.usagePercent)}%"
			></div>
		</div>
		<div class="mt-2 flex justify-between text-xs text-text-muted">
			<span>{formatHours(data.yearlyHours)} / {formatHours(data.contractHours)}</span>
			<span>{data.yearlySessions} 個工作階段</span>
		</div>
	</div>
{/if}

<!-- Monthly Chart -->
<div class="mb-8 rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-6 text-lg font-semibold text-text">月度工時趨勢</h2>
	<div class="flex h-48 items-end justify-between gap-2">
		{#each data.monthlyStats as stat, i}
			{@const heightPercent = maxHours > 0 ? (stat.hours / maxHours) * 100 : 0}
			{@const isCurrentMonth = i === new Date().getMonth()}
			<div class="group flex flex-1 flex-col items-center">
				<div class="relative mb-2 w-full">
					<div
						class="mx-auto w-full max-w-8 rounded-t transition-all {isCurrentMonth
							? 'bg-primary'
							: 'bg-secondary/60'} group-hover:bg-primary"
						style="height: {Math.max(4, heightPercent * 1.5)}px"
					></div>
					<!-- Tooltip -->
					<div
						class="pointer-events-none absolute bottom-full left-1/2 mb-2 -translate-x-1/2 whitespace-nowrap rounded bg-text px-2 py-1 text-xs text-surface opacity-0 transition-opacity group-hover:opacity-100"
					>
						{formatHours(stat.hours)}
						<br />
						<span class="text-text-muted">{stat.sessions} 階段</span>
					</div>
				</div>
				<span class="text-xs text-text-muted {isCurrentMonth ? 'font-bold text-primary' : ''}">
					{i + 1}月
				</span>
			</div>
		{/each}
	</div>
</div>

<!-- Monthly Reports List -->
<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-6 text-lg font-semibold text-text">月份報告</h2>

	{#if data.monthsWithData.length === 0}
		<div class="py-8 text-center text-text-muted">
			<svg class="mx-auto h-12 w-12 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
				<rect x="3" y="4" width="18" height="18" rx="2" />
				<path d="M16 2v4M8 2v4M3 10h18" />
			</svg>
			<p class="mt-4">本年度尚無工作紀錄</p>
		</div>
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
