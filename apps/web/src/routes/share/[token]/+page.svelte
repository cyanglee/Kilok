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

	<!-- Contract Stats - 簡化版：只顯示當月累積和使用 -->
	{#if data.hasContract && data.monthlyHours > 0}
		{@const currentMonthData = data.monthlyQuotaBreakdown.find(m => m.month === data.currentMonth)}
		{@const currentQuota = currentMonthData ? currentMonthData.startBalance + currentMonthData.quota : data.monthlyHours}
		{@const currentUsed = currentMonthData?.used ?? 0}
		{@const currentRemaining = currentQuota - currentUsed}
		<div class="mb-8 grid grid-cols-2 gap-4">
			<!-- 當月可用額度 -->
			<div class="rounded-xl border border-border bg-surface p-5 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">{data.currentMonth}月可用額度</div>
				<div class="mt-1 text-2xl font-bold text-secondary">
					{formatHours(currentQuota)}
				</div>
			</div>

			<!-- 當月已使用 -->
			<div class="rounded-xl border-2 border-primary/30 bg-primary/5 p-5 text-center shadow-sm">
				<div class="text-xs font-medium text-primary/70">本月已使用</div>
				<div class="mt-1 text-2xl font-bold text-primary">
					{formatHours(currentUsed)}
				</div>
				<div class="mt-1 text-xs {currentRemaining < data.monthlyHours ? 'text-warning' : 'text-success'}">
					剩餘 {formatHours(currentRemaining)}
				</div>
			</div>
		</div>
	{:else if data.hasContract}
		<!-- 無月額度時顯示年度資訊 -->
		<div class="mb-8 grid grid-cols-2 gap-4">
			<div class="rounded-xl border border-border bg-surface p-5 text-center shadow-sm">
				<div class="text-xs font-medium text-text-muted">年度額度</div>
				<div class="mt-1 text-2xl font-bold text-secondary">{formatHours(data.yearlyQuota)}</div>
			</div>
			<div class="rounded-xl border-2 border-primary/30 bg-primary/5 p-5 text-center shadow-sm">
				<div class="text-xs font-medium text-primary/70">已使用</div>
				<div class="mt-1 text-2xl font-bold text-primary">{formatHours(data.yearlyBillableHours)}</div>
				<div class="mt-1 text-xs text-success">剩餘 {formatHours(data.remainingHours)}</div>
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
