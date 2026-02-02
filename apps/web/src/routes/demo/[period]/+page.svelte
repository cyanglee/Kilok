<script lang="ts">
	import type { PageData } from './$types';
	import { formatHours, formatBillableHours, formatDate, monthNames } from '$lib/utils/formatters';
	import { StatsCard, PeriodNavigation, EmptyState } from '$lib/components';

	let { data }: { data: PageData } = $props();

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

	// 計算月額度剩餘/超額
	const monthlyRemaining = data.monthlyHours - data.completedBillableHours;
	const monthlySubtitle = monthlyRemaining >= 0
		? `剩餘 ${formatHours(monthlyRemaining)}`
		: `超額 ${formatHours(-monthlyRemaining)}`;
</script>

<svelte:head>
	<title>Demo {data.year} {monthNames[data.month - 1]} - Kilok</title>
</svelte:head>

<div class="max-w-5xl mx-auto">
	<!-- Demo Banner -->
	<div class="mb-6 rounded-xl border border-blue-200 bg-blue-50 px-4 py-3">
		<div class="flex items-center gap-3">
			<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-100">
				<svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
			</div>
			<div>
				<p class="text-sm font-medium text-blue-900">這是 Demo 頁面</p>
				<p class="text-xs text-blue-700">顯示的是模擬資料，實際使用時會顯示您的真實工時紀錄</p>
			</div>
		</div>
	</div>

	<!-- Breadcrumb -->
	<nav class="mb-6 flex items-center gap-2 text-sm">
		<a href="/demo" class="text-text-muted hover:text-primary">{data.clientName}</a>
		<span class="text-text-muted">/</span>
		<span class="text-text">{data.year} {monthNames[data.month - 1]}</span>
	</nav>

	<!-- Page Header with Month Navigation -->
	<div class="mb-8">
		<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
			<div>
				<h1 class="text-2xl font-bold text-text">
					{data.year} 年 {monthNames[data.month - 1]} 月報
				</h1>
				<p class="mt-1 text-text-muted">{data.clientName}</p>
			</div>
			<PeriodNavigation
				prevHref="/demo/{data.prevPeriod}"
				nextHref="/demo/{data.nextPeriod}"
			/>
		</div>
	</div>

	<!-- Stats Cards -->
	<div class="mb-8 flex flex-wrap gap-4">
		<StatsCard
			label="本月計費工時"
			value={formatBillableHours(data.completedBillableHours)}
			valueClass="text-primary text-3xl"
		/>
		{#if data.monthlyHours > 0}
			<StatsCard
				label="本月額度"
				value={formatHours(data.monthlyHours)}
				subtitle={monthlySubtitle}
				valueClass="text-secondary text-3xl"
			/>
		{/if}
	</div>

	<!-- Completed Work Items -->
	{#if data.completedWorkItems.length > 0}
		<!-- Mobile: Card Layout -->
		<div class="space-y-4 md:hidden">
			{#each data.completedWorkItems as item}
				{@const parsed = parseDescription(item.description)}
				<div class="rounded-xl border border-border bg-surface p-4 shadow-sm">
					<div class="flex items-start justify-between gap-4">
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2 text-xs text-text-muted mb-1">
								<span class="font-mono">{item.completedDate ? formatDate(item.completedDate) : '-'}</span>
							</div>
							<h4 class="font-medium text-text">{item.title}</h4>
							{#if parsed.summary}
								<p class="mt-1 text-sm text-text-muted">{parsed.summary}</p>
							{/if}
							{#if parsed.subItems.length > 0}
								<ul class="mt-2 space-y-1">
									{#each parsed.subItems as subItem}
										<li class="text-sm text-text-muted">
											<span class="font-mono text-xs text-primary/70">{subItem.date}</span>
											<span class="ml-1">{subItem.title}</span>
											{#if subItem.time}
												<span class="text-xs">({subItem.time})</span>
											{/if}
										</li>
									{/each}
								</ul>
							{/if}
						</div>
						<div class="text-right shrink-0">
							<div class="text-lg font-bold text-primary">{formatBillableHours(item.billableHours)}</div>
						</div>
					</div>
				</div>
			{/each}

			<!-- 合計 -->
			<div class="rounded-xl border border-border bg-background/50 p-4">
				<div class="flex items-center justify-between">
					<span class="font-medium text-text">合計</span>
					<span class="text-xl font-bold text-primary">{formatBillableHours(data.completedBillableHours)}</span>
				</div>
			</div>
		</div>

		<!-- Desktop: Table Layout -->
		<div class="hidden md:block overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
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
							<td class="px-6 py-4 text-sm text-text-muted font-mono whitespace-nowrap">
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
							<td class="px-6 py-4 text-right whitespace-nowrap">
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
		<EmptyState message="本月尚無已完成的工作報告" />
	{/if}

	<!-- Back link -->
	<div class="mt-8 text-center">
		<a
			href="/demo"
			class="inline-flex items-center gap-2 text-sm text-text-muted hover:text-primary transition-colors"
		>
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
			</svg>
			返回年度報告
		</a>
	</div>
</div>
