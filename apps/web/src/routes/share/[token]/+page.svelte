<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	function formatHours(hours: number): string {
		const h = Math.floor(hours);
		const m = Math.round((hours - h) * 60);
		return m > 0 ? `${h}h ${m}m` : `${h}h`;
	}

	function formatBillableHours(hours: number): string {
		if (hours % 1 === 0.5) {
			return `${hours}h`;
		}
		return `${Math.floor(hours)}h`;
	}

	function formatPeriod(period: string): string {
		const [year, month] = period.split('-');
		const monthNames = [
			'一月', '二月', '三月', '四月', '五月', '六月',
			'七月', '八月', '九月', '十月', '十一月', '十二月'
		];
		return `${year} 年 ${monthNames[parseInt(month, 10) - 1]}`;
	}

	// 建立完整 12 個月的圖表資料
	const monthlyChartData = Array.from({ length: 12 }, (_, i) => {
		const month = i + 1;
		const stat = data.monthlyStats.find(s => s.month === month && s.year === data.currentYear);
		return {
			month,
			billableHours: stat?.billableHours ?? 0
		};
	});

	const maxBillableHours = Math.max(...monthlyChartData.map(m => m.billableHours), 1);
</script>

<div class="max-w-5xl mx-auto">
	<!-- Header -->
	<div class="mb-8 text-center">
		<h1 class="text-2xl font-bold text-text">{data.clientName}</h1>
		<p class="mt-2 text-text-muted">{data.currentYear} 年度工時報告</p>
	</div>

	<!-- Contract Stats -->
	{#if data.hasContract}
		<!-- 年度統計卡片 -->
		<div class="mb-8 grid grid-cols-2 gap-4 md:grid-cols-4">
			<!-- 年度總額度 -->
			<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">年度額度</div>
				<div class="mt-1 text-xl font-bold text-secondary">
					{formatHours(data.yearlyQuota)}
				</div>
				{#if data.monthlyHours > 0}
					<div class="mt-1 text-xs text-text-muted">
						12月 × {formatHours(data.monthlyHours)}{#if data.carriedOver > 0}&nbsp;+ {formatHours(data.carriedOver)} 結轉{/if}
					</div>
				{/if}
			</div>

			<!-- 年度已使用 -->
			<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">年度已使用</div>
				<div class="mt-1 text-xl font-bold text-primary">
					{formatHours(data.yearlyBillableHours)}
				</div>
				<div class="mt-1 text-xs text-text-muted">
					{data.usagePercent.toFixed(0)}%
				</div>
			</div>

			<!-- 年度剩餘額度 -->
			<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">年度剩餘</div>
				<div class="mt-1 text-xl font-bold {data.remainingHours < data.monthlyHours * 3 ? 'text-warning' : 'text-success'}">
					{formatHours(data.remainingHours)}
				</div>
			</div>

			<!-- 結轉時數 -->
			{#if data.carriedOver > 0}
				<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
					<div class="text-xs font-medium text-text-muted">年度結轉</div>
					<div class="mt-1 text-xl font-bold text-cyan-600">
						+{formatHours(data.carriedOver)}
					</div>
				</div>
			{/if}
		</div>

		<!-- 年度進度條 -->
		<div class="mb-8 rounded-xl border border-border bg-surface p-5 shadow-sm">
			<div class="mb-2 flex justify-between">
				<span class="text-sm font-medium text-text">年度額度使用進度</span>
				<span class="text-sm font-bold {data.usagePercent > 90 ? 'text-danger' : data.usagePercent > 70 ? 'text-warning' : 'text-text'}">
					{data.usagePercent.toFixed(1)}%
				</span>
			</div>
			<div class="h-3 overflow-hidden rounded-full bg-border">
				<div
					class="h-full rounded-full transition-all {data.usagePercent > 90 ? 'bg-danger' : data.usagePercent > 70 ? 'bg-warning' : 'bg-primary'}"
					style="width: {Math.min(100, data.usagePercent)}%"
				></div>
			</div>
			<div class="mt-2 flex justify-between text-xs text-text-muted">
				<span>{formatHours(data.yearlyBillableHours)} / {formatHours(data.yearlyQuota)}</span>
				<span>剩餘 {formatHours(data.remainingHours)}</span>
			</div>
		</div>
	{/if}

	<!-- Monthly Chart -->
	<div class="mb-8 rounded-xl border border-border bg-surface p-6 shadow-sm">
		<h2 class="mb-6 text-lg font-semibold text-text">月度工時趨勢</h2>
		<div class="flex h-48 items-end justify-between gap-2">
			{#each monthlyChartData as stat, i}
				{@const heightPercent = maxBillableHours > 0 ? (stat.billableHours / maxBillableHours) * 100 : 0}
				{@const isCurrentMonth = i === data.currentMonth - 1}
				<div class="group flex flex-1 flex-col items-center">
					<div class="relative mb-2 w-full">
						<div
							class="mx-auto w-full max-w-8 rounded-t transition-all {isCurrentMonth
								? 'bg-primary'
								: 'bg-secondary/60'} group-hover:bg-primary"
							style="height: {Math.max(4, heightPercent * 1.5)}px"
						></div>
						<!-- Tooltip -->
						{#if stat.billableHours > 0}
							<div
								class="pointer-events-none absolute bottom-full left-1/2 mb-2 -translate-x-1/2 whitespace-nowrap rounded bg-text px-2 py-1 text-xs text-surface opacity-0 transition-opacity group-hover:opacity-100"
							>
								{formatBillableHours(stat.billableHours)}
							</div>
						{/if}
					</div>
					<span class="text-xs text-text-muted {isCurrentMonth ? 'font-bold text-primary' : ''}">
						{i + 1}月
					</span>
				</div>
			{/each}
		</div>
	</div>

	<!-- Monthly List -->
	<div class="rounded-xl border border-border bg-surface shadow-sm overflow-hidden">
		<div class="border-b border-border bg-background/50 px-6 py-3">
			<h2 class="text-sm font-semibold text-text">月份報告</h2>
		</div>
		{#if data.monthlyStats.length > 0}
			<div class="divide-y divide-border">
				{#each data.monthlyStats as stat}
					<a
						href="/share/{data.token}/{stat.period}"
						class="flex items-center justify-between px-6 py-4 hover:bg-background/50 transition-colors"
					>
						<span class="font-medium text-text">{formatPeriod(stat.period)}</span>
						<span class="text-primary font-bold">{formatBillableHours(stat.billableHours)}</span>
					</a>
				{/each}
			</div>
		{:else}
			<div class="p-12 text-center">
				<p class="text-text-muted">尚無工時紀錄</p>
			</div>
		{/if}
	</div>
</div>
