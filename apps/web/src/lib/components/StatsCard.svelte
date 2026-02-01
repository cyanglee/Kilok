<script lang="ts">
	interface Props {
		label: string;
		value: string;
		subtitle?: string;
		valueClass?: string;
		highlighted?: boolean;
	}

	let { label, value, subtitle, valueClass = 'text-text', highlighted = false }: Props = $props();
</script>

<div
	class="rounded-xl border bg-surface p-4 text-center shadow-sm transition-shadow hover:shadow-md
		{highlighted ? 'border-2 border-primary/30 bg-primary/5' : 'border-border'}"
>
	<div class="text-xs font-medium {highlighted ? 'text-primary/70' : 'text-text-muted'}">{label}</div>
	<div class="mt-1 text-xl font-bold {valueClass}">{value}</div>
	{#if subtitle}
		<div class="mt-1 text-xs text-text-muted">{subtitle}</div>
	{/if}
</div>
