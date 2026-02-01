<script lang="ts">
	interface Props {
		prevHref: string;
		nextHref: string;
		prevLabel?: string;
		nextLabel?: string;
	}

	let {
		prevHref,
		nextHref,
		prevLabel = '← 上月',
		nextLabel = '下月 →'
	}: Props = $props();
</script>

<div class="flex items-center gap-2">
	<a
		href={prevHref}
		class="whitespace-nowrap rounded-lg border border-border px-3 py-2 text-sm text-text-muted hover:bg-surface hover:text-text transition-colors"
	>
		{prevLabel}
	</a>
	<a
		href={nextHref}
		class="whitespace-nowrap rounded-lg border border-border px-3 py-2 text-sm text-text-muted hover:bg-surface hover:text-text transition-colors"
	>
		{nextLabel}
	</a>
</div>
