<script lang="ts">
	import { formatHours, formatBillableHours } from '$lib/utils/formatters';

	interface MonthData {
		month: number;
		hours?: number;
		billableHours?: number;
		sessions?: number;
	}

	interface Props {
		title?: string;
		data: MonthData[];
		currentMonth: number;
		useBillableHours?: boolean;
		showSessions?: boolean;
	}

	let {
		title = '月度工時趨勢',
		data,
		currentMonth,
		useBillableHours = false,
		showSessions = false
	}: Props = $props();

	const getValue = (item: MonthData) => useBillableHours ? (item.billableHours ?? 0) : (item.hours ?? 0);
	const formatValue = useBillableHours ? formatBillableHours : formatHours;
	const maxValue = Math.max(...data.map(getValue), 1);
</script>

<div class="rounded-xl border border-border bg-surface p-6 shadow-sm">
	<h2 class="mb-6 text-lg font-semibold text-text">{title}</h2>
	<div class="flex h-48 items-end justify-between gap-2">
		{#each data as stat, i}
			{@const value = getValue(stat)}
			{@const heightPercent = maxValue > 0 ? (value / maxValue) * 100 : 0}
			{@const isCurrentMonth = i === currentMonth - 1}
			<div class="group flex flex-1 flex-col items-center">
				<div class="relative mb-2 w-full">
					<div
						class="mx-auto w-full max-w-8 rounded-t transition-all {isCurrentMonth
							? 'bg-primary'
							: 'bg-secondary/60'} group-hover:bg-primary"
						style="height: {Math.max(4, heightPercent * 1.5)}px"
					></div>
					<!-- Tooltip -->
					{#if value > 0}
						<div
							class="pointer-events-none absolute bottom-full left-1/2 mb-2 -translate-x-1/2 whitespace-nowrap rounded bg-text px-2 py-1 text-xs text-surface opacity-0 transition-opacity group-hover:opacity-100"
						>
							{formatValue(value)}
							{#if showSessions && stat.sessions}
								<br />
								<span class="text-text-muted">{stat.sessions} 階段</span>
							{/if}
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
