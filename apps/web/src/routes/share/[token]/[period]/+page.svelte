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

	function formatDate(dateStr: string): string {
		const parts = dateStr.split('-');
		if (parts.length === 3) {
			return `${parts[1]}/${parts[2]}`;
		}
		return dateStr;
	}

	const monthNames = [
		'一月', '二月', '三月', '四月', '五月', '六月',
		'七月', '八月', '九月', '十月', '十一月', '十二月'
	];

	// Parse description for sub-items with dates and time
	interface SubItem {
		date: string;
		title: string;
		time: string | null;
		description: string;
	}

	function parseDescription(description: string | null): { summary: string | null; subItems: SubItem[] } {
		if (!description) return { summary: null, subItems: [] };

		const lines = description.split('\n').filter((l) => l.trim());
		const subItems: SubItem[] = [];
		const otherLines: string[] = [];

		for (const line of lines) {
			const match4 = line.match(/^(\d{4}\/\d{2}\/\d{2})\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*(.+)$/);
			if (match4) {
				subItems.push({
					date: match4[1],
					title: match4[2].trim(),
					time: match4[3].trim(),
					description: match4[4].trim()
				});
				continue;
			}
			const match3 = line.match(/^(\d{4}\/\d{2}\/\d{2})\s*\|\s*([^|]+)\s*\|\s*(.+)$/);
			if (match3) {
				subItems.push({
					date: match3[1],
					title: match3[2].trim(),
					time: null,
					description: match3[3].trim()
				});
				continue;
			}
			otherLines.push(line);
		}

		return {
			summary: otherLines.length > 0 ? otherLines.join('\n') : null,
			subItems
		};
	}
</script>

<div class="max-w-3xl mx-auto">
	<!-- Breadcrumb -->
	<nav class="mb-6 flex items-center gap-2 text-sm">
		<a href="/share/{data.token}" class="text-text-muted hover:text-primary">{data.clientName}</a>
		<span class="text-text-muted">/</span>
		<span class="text-text">{data.year} {monthNames[data.month - 1]}</span>
	</nav>

	<!-- Page Header with Month Navigation -->
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-text">
				{data.year} 年 {monthNames[data.month - 1]} 月報
			</h1>
			<p class="mt-1 text-text-muted">{data.clientName}</p>
		</div>
		<div class="flex items-center gap-2">
			<a
				href="/share/{data.token}/{data.prevPeriod}"
				class="rounded-lg border border-border px-3 py-2 text-sm text-text-muted hover:bg-surface hover:text-text transition-colors"
			>
				← 上月
			</a>
			<a
				href="/share/{data.token}/{data.nextPeriod}"
				class="rounded-lg border border-border px-3 py-2 text-sm text-text-muted hover:bg-surface hover:text-text transition-colors"
			>
				下月 →
			</a>
		</div>
	</div>

	<!-- Stats Card -->
	<div class="mb-8">
		<div
			class="inline-flex items-center gap-4 rounded-xl border border-border bg-surface px-6 py-4 shadow-sm"
		>
			<div>
				<div class="text-sm font-medium text-text-muted">本月計費工時</div>
				<div class="text-3xl font-bold text-primary">{formatBillableHours(data.completedBillableHours)}</div>
			</div>
		</div>
	</div>

	<!-- Completed Work Items Table -->
	{#if data.completedWorkItems.length > 0}
		<div class="overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
			<table class="w-full">
				<thead>
					<tr class="border-b border-border bg-background/50">
						<th class="px-6 py-3 text-left text-xs font-medium text-text-muted w-24 whitespace-nowrap">完成日期</th>
						<th class="px-6 py-3 text-left text-xs font-medium text-text-muted">工作項目</th>
						<th class="px-6 py-3 text-right text-xs font-medium text-text-muted w-24">計費工時</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-border">
					{#each data.completedWorkItems as item}
						{@const parsed = parseDescription(item.description)}
						<tr>
							<td class="px-6 py-4 text-sm text-text-muted font-mono">
								{item.completedDate ? formatDate(item.completedDate) : '-'}
							</td>
							<td class="px-6 py-4">
								<div class="min-w-0">
									<h4 class="font-medium text-text">{item.title}</h4>
									{#if parsed.summary}
										<p class="mt-1 text-sm text-text-muted">{parsed.summary}</p>
									{/if}
									{#if parsed.subItems.length > 0}
										<ul class="mt-2 space-y-1.5">
											{#each parsed.subItems as subItem}
												<li class="text-sm">
													<span class="text-primary/70 font-mono text-xs">{subItem.date}</span>
													<span class="font-medium text-text ml-2">{subItem.title}</span>
													{#if subItem.time}
														<span class="text-primary/80 text-xs ml-1">({subItem.time})</span>
													{/if}
													<span class="text-text-muted ml-1">- {subItem.description}</span>
												</li>
											{/each}
										</ul>
									{/if}
								</div>
							</td>
							<td class="px-6 py-4 text-right">
								<div class="text-lg font-bold text-primary">{formatBillableHours(item.billableHours)}</div>
							</td>
						</tr>
					{/each}
				</tbody>
				<tfoot>
					<tr class="border-t border-border bg-background/50">
						<td class="px-6 py-3"></td>
						<td class="px-6 py-3 text-sm font-medium text-text">合計</td>
						<td class="px-6 py-3 text-right">
							<div class="text-lg font-bold text-primary">{formatBillableHours(data.completedBillableHours)}</div>
						</td>
					</tr>
				</tfoot>
			</table>
		</div>
	{:else}
		<div class="rounded-xl border border-border bg-surface p-12 text-center shadow-sm">
			<p class="text-text-muted">本月尚無已完成的工作報告</p>
		</div>
	{/if}
</div>
