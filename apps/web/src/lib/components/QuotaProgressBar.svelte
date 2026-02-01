<script lang="ts">
	import { formatHours } from '$lib/utils/formatters';

	interface Props {
		label: string;
		current: number;
		quota: number;
		leftText?: string;
		rightText?: string;
		warningThreshold?: number;
		dangerThreshold?: number;
	}

	let {
		label,
		current,
		quota,
		leftText,
		rightText,
		warningThreshold = 70,
		dangerThreshold = 90
	}: Props = $props();

	const percentage = quota > 0 ? (current / quota) * 100 : 0;
	const colorClass = percentage > dangerThreshold
		? 'text-danger'
		: percentage > warningThreshold
			? 'text-warning'
			: 'text-text';
	const barColorClass = percentage > dangerThreshold
		? 'bg-danger'
		: percentage > warningThreshold
			? 'bg-warning'
			: 'bg-primary';
</script>

<div class="rounded-xl border border-border bg-surface p-5 shadow-sm">
	<div class="mb-2 flex justify-between">
		<span class="text-sm font-medium text-text">{label}</span>
		<span class="text-sm font-bold {colorClass}">
			{percentage.toFixed(1)}%
		</span>
	</div>
	<div class="h-3 overflow-hidden rounded-full bg-border">
		<div
			class="h-full rounded-full transition-all {barColorClass}"
			style="width: {Math.min(100, percentage)}%"
		></div>
	</div>
	<div class="mt-2 flex justify-between text-xs text-text-muted">
		<span>{leftText ?? `${formatHours(current)} / ${formatHours(quota)}`}</span>
		<span>{rightText ?? `剩餘 ${formatHours(Math.max(0, quota - current))}`}</span>
	</div>
</div>
