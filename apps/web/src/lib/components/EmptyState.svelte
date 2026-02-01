<script lang="ts">
	interface Props {
		message: string;
		showIcon?: boolean;
	}

	let { message, showIcon = true }: Props = $props();
</script>

<div class="rounded-xl border border-border bg-surface p-12 text-center shadow-sm">
	{#if showIcon}
		<svg class="mx-auto h-12 w-12 opacity-50 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
			<rect x="3" y="4" width="18" height="18" rx="2" />
			<path d="M16 2v4M8 2v4M3 10h18" />
		</svg>
	{/if}
	<p class="{showIcon ? 'mt-4' : ''} text-text-muted">{message}</p>
</div>
