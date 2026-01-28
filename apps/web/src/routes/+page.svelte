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
</script>

<!-- Page Header -->
<div class="mb-8">
	<h1 class="text-3xl font-bold text-text">工時總覽</h1>
	<p class="mt-1 text-text-muted">
		{data.currentYear} 年 {monthNames[data.currentMonth - 1]}
	</p>
</div>

<!-- Bento Grid: Summary Cards -->
<div class="mb-8 grid grid-cols-1 gap-4 md:grid-cols-3">
	<!-- Total Monthly Hours -->
	<div
		class="rounded-xl border border-border bg-surface p-6 shadow-sm transition-shadow hover:shadow-md"
	>
		<div class="text-sm font-medium text-text-muted">本月工時</div>
		<div class="mt-2 text-3xl font-bold text-primary">{formatHours(data.totalMonthlyHours)}</div>
	</div>

	<!-- Total Yearly Hours -->
	<div
		class="rounded-xl border border-border bg-surface p-6 shadow-sm transition-shadow hover:shadow-md"
	>
		<div class="text-sm font-medium text-text-muted">年度累計</div>
		<div class="mt-2 text-3xl font-bold text-text">{formatHours(data.totalYearlyHours)}</div>
	</div>

	<!-- Active Projects -->
	<div
		class="rounded-xl border border-border bg-surface p-6 shadow-sm transition-shadow hover:shadow-md"
	>
		<div class="text-sm font-medium text-text-muted">追蹤專案</div>
		<div class="mt-2 text-3xl font-bold text-secondary">{data.activeProjects}</div>
	</div>
</div>

<!-- Client List -->
<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-6 text-xl font-semibold text-text">客戶列表</h2>

	{#if data.clientStats.length === 0}
		<div class="py-12 text-center text-text-muted">
			<svg class="mx-auto h-12 w-12 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
				<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
				<circle cx="12" cy="7" r="4" />
			</svg>
			<p class="mt-4">尚未設定客戶</p>
			<p class="mt-1 text-sm">請先在 CLI 中設定客戶和專案關聯</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			{#each data.clientStats as stat}
				<a
					href="/{stat.client.slug}"
					class="group cursor-pointer rounded-lg border border-border p-5 transition-all hover:border-primary hover:shadow-md"
				>
					<div class="flex items-start justify-between">
						<div>
							<h3 class="font-semibold text-text group-hover:text-primary">{stat.client.name}</h3>
							<p class="mt-1 text-sm text-text-muted">
								{stat.projects.length} 個專案
							</p>
						</div>
						<div class="text-right">
							<div class="text-lg font-bold text-text">{formatHours(stat.monthlyHours)}</div>
							<div class="text-xs text-text-muted">本月</div>
						</div>
					</div>

					{#if stat.contractHours > 0}
						<!-- Progress Bar -->
						<div class="mt-4">
							<div class="mb-1 flex justify-between text-xs">
								<span class="text-text-muted">年度額度使用</span>
								<span
									class={stat.usagePercent > 90
										? 'text-danger'
										: stat.usagePercent > 70
											? 'text-warning'
											: 'text-text-muted'}
								>
									{stat.usagePercent.toFixed(0)}%
								</span>
							</div>
							<div class="h-2 overflow-hidden rounded-full bg-border">
								<div
									class="h-full rounded-full transition-all {stat.usagePercent > 90
										? 'bg-danger'
										: stat.usagePercent > 70
											? 'bg-warning'
											: 'bg-primary'}"
									style="width: {Math.min(100, stat.usagePercent)}%"
								></div>
							</div>
							<div class="mt-1 flex justify-between text-xs text-text-muted">
								<span>{formatHours(stat.yearlyHours)} 已使用</span>
								<span>剩餘 {formatHours(stat.remainingHours)}</span>
							</div>
						</div>
					{:else}
						<div class="mt-4 text-xs text-text-muted">
							年度累計：{formatHours(stat.yearlyHours)}
						</div>
					{/if}
				</a>
			{/each}
		</div>
	{/if}
</div>

<!-- Unassigned Projects -->
{#if data.unassignedStats && data.unassignedStats.length > 0}
	<div class="mt-8 rounded-xl border border-border bg-surface p-6 shadow-sm">
		<details class="group">
			<summary class="flex cursor-pointer list-none items-center justify-between">
				<h2 class="text-xl font-semibold text-text">未分配專案</h2>
				<div class="flex items-center gap-3">
					<span class="text-sm text-text-muted">{data.unassignedStats.length} 個專案</span>
					<svg
						class="h-5 w-5 text-text-muted transition-transform group-open:rotate-180"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<path d="M6 9l6 6 6-6" />
					</svg>
				</div>
			</summary>

			<div class="mt-4 grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
				{#each data.unassignedStats as stat}
					<div class="rounded-lg border border-border p-4 transition-all hover:border-secondary">
						<div class="flex items-start justify-between">
							<div class="min-w-0 flex-1">
								<h3 class="truncate font-medium text-text" title={stat.project.path}>
									{stat.project.display_name || stat.project.path.split('/').pop()}
								</h3>
								<p class="mt-0.5 truncate text-xs text-text-muted" title={stat.project.path}>
									{stat.project.path}
								</p>
							</div>
							<div class="ml-3 text-right">
								<div class="font-semibold text-text">{formatHours(stat.monthlyHours)}</div>
								<div class="text-xs text-text-muted">本月</div>
							</div>
						</div>
						{#if stat.yearlyHours > 0}
							<div class="mt-2 text-xs text-text-muted">
								年度累計：{formatHours(stat.yearlyHours)}
							</div>
						{/if}
					</div>
				{/each}
			</div>

			<p class="mt-4 text-sm text-text-muted">
				這些專案尚未分配給任何客戶。前往
				<a href="/admin/projects" class="text-primary underline hover:text-primary/80">管理專案</a>
				進行分配。
			</p>
		</details>
	</div>
{/if}
