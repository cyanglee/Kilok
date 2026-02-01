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
</script>

<div class="max-w-2xl mx-auto">
	<!-- Header -->
	<div class="mb-8 text-center">
		<h1 class="text-2xl font-bold text-text">{data.clientName}</h1>
		<p class="mt-2 text-text-muted">{data.currentYear} 年度工時報告</p>
	</div>

	<!-- Contract Stats -->
	{#if data.hasContract}
		<div class="mb-8 grid grid-cols-2 gap-4 md:grid-cols-4">
			<!-- 年度總額度 -->
			<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">年度額度</div>
				<div class="mt-1 text-xl font-bold text-secondary">
					{formatHours(data.yearlyQuota)}
				</div>
				{#if data.monthlyHours > 0}
					<div class="mt-1 text-xs text-text-muted">
						12月 × {formatHours(data.monthlyHours)}
						{#if data.carriedOver > 0}+ 結轉{/if}
					</div>
				{/if}
			</div>

			<!-- 已使用 -->
			<div class="rounded-xl border border-border bg-surface p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">已使用</div>
				<div class="mt-1 text-xl font-bold text-primary">
					{formatHours(data.yearlyBillableHours)}
				</div>
				<div class="mt-1 text-xs text-text-muted">
					{data.usagePercent.toFixed(0)}%
				</div>
			</div>

			<!-- 剩餘額度 -->
			<div class="rounded-xl border-2 border-primary/30 bg-primary/5 p-4 text-center shadow-sm">
				<div class="text-xs font-medium text-primary/70">剩餘額度</div>
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

		<!-- Progress Bar -->
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

	<!-- Monthly Quota Breakdown -->
	{#if data.monthlyQuotaBreakdown.length > 0}
		<div class="mb-8 rounded-xl border border-border bg-surface shadow-sm overflow-hidden">
			<div class="border-b border-border bg-background/50 px-6 py-3">
				<h2 class="text-sm font-semibold text-text">月度額度明細</h2>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b border-border bg-background/30">
							<th class="px-4 py-3 text-left font-medium text-text-muted">月份</th>
							<th class="px-4 py-3 text-right font-medium text-text-muted">月初餘額</th>
							<th class="px-4 py-3 text-right font-medium text-text-muted">+額度</th>
							<th class="px-4 py-3 text-right font-medium text-text-muted">-使用</th>
							<th class="px-4 py-3 text-right font-medium text-text-muted">月底餘額</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-border">
						{#if data.carriedOver > 0}
							<tr class="bg-cyan-50/50">
								<td class="px-4 py-2 text-text-muted">上年度結轉</td>
								<td class="px-4 py-2 text-right">-</td>
								<td class="px-4 py-2 text-right text-cyan-600 font-medium">+{formatHours(data.carriedOver)}</td>
								<td class="px-4 py-2 text-right">-</td>
								<td class="px-4 py-2 text-right font-medium">{formatHours(data.carriedOver)}</td>
							</tr>
						{/if}
						{#each data.monthlyQuotaBreakdown as item}
							{@const isCurrent = item.month === data.currentMonth}
							{@const isFuture = item.month > data.currentMonth}
							<tr class="{isCurrent ? 'bg-primary/5' : ''} {isFuture ? 'text-text-muted/60' : ''}">
								<td class="px-4 py-2 {isCurrent ? 'font-bold text-primary' : 'text-text'}">
									{item.month}月
									{#if isCurrent}
										<span class="ml-1 text-xs text-primary/70">(本月)</span>
									{/if}
								</td>
								<td class="px-4 py-2 text-right">{formatHours(item.startBalance)}</td>
								<td class="px-4 py-2 text-right text-success">+{formatHours(item.quota)}</td>
								<td class="px-4 py-2 text-right {item.used > 0 ? 'text-primary font-medium' : ''}">
									{#if item.used > 0}
										-{formatHours(item.used)}
									{:else}
										-
									{/if}
								</td>
								<td class="px-4 py-2 text-right font-medium {item.endBalance < item.quota ? 'text-warning' : 'text-success'}">
									{formatHours(item.endBalance)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}

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
