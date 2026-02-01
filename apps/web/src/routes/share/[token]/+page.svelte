<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

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
		<p class="mt-2 text-text-muted">工時報告</p>
	</div>

	<!-- Monthly List -->
	{#if data.monthlyStats.length > 0}
		<div class="overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
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
		</div>
	{:else}
		<div class="rounded-xl border border-border bg-surface p-12 text-center shadow-sm">
			<p class="text-text-muted">尚無工時紀錄</p>
		</div>
	{/if}
</div>
