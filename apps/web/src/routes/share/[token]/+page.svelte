<script lang="ts">
	import type { PageData } from './$types';
	import { formatHours, formatBillableHours, formatPeriod } from '$lib/utils/formatters';
	import { StatsCard, QuotaProgressBar, MonthlyChart, EmptyState } from '$lib/components';

	let { data }: { data: PageData } = $props();

	// 建立完整 12 個月的圖表資料
	const monthlyChartData = Array.from({ length: 12 }, (_, i) => {
		const month = i + 1;
		const stat = data.monthlyStats.find(s => s.month === month && s.year === data.currentYear);
		return {
			month,
			billableHours: stat?.billableHours ?? 0
		};
	});

	// 計算年度額度說明文字
	const yearlyQuotaSubtitle = data.monthlyHours > 0
		? `12月 × ${formatHours(data.monthlyHours)}${data.carriedOver > 0 ? ` + ${formatHours(data.carriedOver)} 結轉` : ''}`
		: undefined;
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
			<StatsCard
				label="年度額度"
				value={formatHours(data.yearlyQuota)}
				subtitle={yearlyQuotaSubtitle}
				valueClass="text-secondary"
			/>
			<StatsCard
				label="年度已使用"
				value={formatHours(data.yearlyBillableHours)}
				subtitle="{data.usagePercent.toFixed(0)}%"
				valueClass="text-primary"
			/>
			<StatsCard
				label="年度剩餘"
				value={formatHours(data.remainingHours)}
				valueClass={data.remainingHours < data.monthlyHours * 3 ? 'text-warning' : 'text-success'}
			/>
			{#if data.carriedOver > 0}
				<StatsCard
					label="年度結轉"
					value="+{formatHours(data.carriedOver)}"
					valueClass="text-cyan-600"
				/>
			{/if}
		</div>

		<!-- 年度進度條 -->
		<div class="mb-8">
			<QuotaProgressBar
				label="年度額度使用進度"
				current={data.yearlyBillableHours}
				quota={data.yearlyQuota}
			/>
		</div>
	{/if}

	<!-- Monthly Chart -->
	<div class="mb-8">
		<MonthlyChart
			data={monthlyChartData}
			currentMonth={data.currentMonth}
			useBillableHours={true}
		/>
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
			<EmptyState message="尚無工時紀錄" showIcon={false} />
		{/if}
	</div>
</div>
