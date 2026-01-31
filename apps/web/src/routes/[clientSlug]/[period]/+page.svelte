<script lang="ts">
	import type { PageData } from './$types';
	import { enhance } from '$app/forms';

	let { data }: { data: PageData } = $props();

	function formatHours(hours: number): string {
		const h = Math.floor(hours);
		const m = Math.round((hours - h) * 60);
		return m > 0 ? `${h}h ${m}m` : `${h}h`;
	}

	function formatSeconds(seconds: number): string {
		return formatHours(seconds / 3600);
	}

	const monthNames = [
		'一月',
		'二月',
		'三月',
		'四月',
		'五月',
		'六月',
		'七月',
		'八月',
		'九月',
		'十月',
		'十一月',
		'十二月'
	];

	// Edit state for work items
	let editingWorkItemId: number | null = $state(null);
	let editTitle = $state('');
	let editDescription = $state('');
	let isSaving = $state(false);

	function startEditing(item: (typeof data.workItemStats)[0]) {
		if (!item.workItemId) return;
		editingWorkItemId = item.workItemId;
		editTitle = item.title ?? '';
		editDescription = item.description ?? '';
	}

	// Parse description for sub-items with dates and time
	// Format: "YYYY/MM/DD | Title | Time | Description" per line
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
			// Check if line matches "YYYY/MM/DD | Title | Time | Description" format (4 parts)
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
			// Fallback: "YYYY/MM/DD | Title | Description" format (3 parts, no time)
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

	function cancelEditing() {
		editingWorkItemId = null;
		editTitle = '';
		editDescription = '';
	}
</script>

<!-- Breadcrumb -->
<nav class="mb-6 flex items-center gap-2 text-sm">
	<a href="/" class="text-text-muted hover:text-primary">首頁</a>
	<span class="text-text-muted">/</span>
	<a href="/{data.client.slug}" class="text-text-muted hover:text-primary">{data.client.name}</a>
	<span class="text-text-muted">/</span>
	<span class="text-text">{data.year} {monthNames[data.month - 1]}</span>
</nav>

<!-- Page Header -->
<div class="mb-8 flex items-start justify-between">
	<div>
		<h1 class="text-3xl font-bold text-text">
			{data.year} 年 {monthNames[data.month - 1]} 月報
		</h1>
		<p class="mt-1 text-text-muted">{data.client.name}</p>
	</div>
</div>

<!-- Stats Card - 只顯示總工時 -->
<div class="mb-8">
	<div
		class="inline-flex items-center gap-3 rounded-xl border border-border bg-surface px-6 py-4 shadow-sm"
	>
		<div class="text-sm font-medium text-text-muted">本月總工時</div>
		<div class="text-3xl font-bold text-primary">{formatHours(data.totalHours)}</div>
	</div>
</div>

<!-- Work Items Table - 簡潔的業主報告 -->
{#if data.workItemStats.length > 0}
	<div class="overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
		<table class="w-full">
			<thead>
				<tr class="border-b border-border bg-background/50">
					<th class="px-6 py-3 text-left text-sm font-medium text-text-muted">工作項目</th>
					<th class="px-6 py-3 text-right text-sm font-medium text-text-muted">工時</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-border">
				{#each data.workItemStats as item}
					<tr class="group hover:bg-background/30 transition-colors">
						<td class="px-6 py-4">
							{#if editingWorkItemId === item.workItemId}
								<!-- Edit Mode -->
								<form
									method="POST"
									action="?/updateWorkItem"
									use:enhance={() => {
										isSaving = true;
										return async ({ result, update }) => {
											isSaving = false;
											if (result.type === 'success') {
												cancelEditing();
											}
											await update();
										};
									}}
									class="space-y-2"
								>
									<input type="hidden" name="id" value={item.workItemId} />
									<input
										type="text"
										name="title"
										placeholder="工作項目名稱"
										bind:value={editTitle}
										class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text placeholder:text-text-muted focus:border-primary focus:outline-none"
									/>
									<textarea
										name="description"
										placeholder="工作內容說明"
										bind:value={editDescription}
										rows="2"
										class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text placeholder:text-text-muted focus:border-primary focus:outline-none"
									></textarea>
									<div class="flex gap-2">
										<button
											type="submit"
											disabled={isSaving}
											class="rounded-lg bg-primary px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-primary/90 disabled:opacity-50"
										>
											{isSaving ? '儲存中...' : '儲存'}
										</button>
										<button
											type="button"
											onclick={cancelEditing}
											class="rounded-lg border border-border px-3 py-1.5 text-xs font-medium text-text-muted transition-colors hover:bg-border"
										>
											取消
										</button>
									</div>
								</form>
							{:else}
								<!-- Display Mode -->
								{@const parsed = parseDescription(item.description)}
								<div class="flex items-start justify-between">
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-2">
											{#if item.title}
												<h4 class="font-medium text-text">{item.title}</h4>
											{:else}
												<h4 class="font-medium text-text-muted italic">
													{item.workItem}
												</h4>
											{/if}
											{#if item.lastDate}
												<span class="text-xs text-text-muted font-mono">({item.lastDate})</span>
											{/if}
										</div>
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
										{#if item.commits.length > 0}
											<details class="mt-2 group/commits">
												<summary class="text-xs text-text-muted cursor-pointer hover:text-primary">
													{item.commits.length} 個 commits
												</summary>
												<ul class="mt-1 space-y-0.5 pl-2 border-l-2 border-border">
													{#each item.commits as commit}
														<li class="text-xs text-text-muted">
															<code class="text-primary/60">{commit.hash.slice(0, 7)}</code>
															<span class="ml-1">{commit.message ?? '(no message)'}</span>
														</li>
													{/each}
												</ul>
											</details>
										{/if}
									</div>
									{#if item.workItemId}
										<button
											onclick={() => startEditing(item)}
											class="ml-2 opacity-0 group-hover:opacity-100 transition-opacity text-xs text-primary hover:text-primary/80"
										>
											編輯
										</button>
									{/if}
								</div>
							{/if}
						</td>
						<td class="px-6 py-4 text-right">
							<span class="text-lg font-bold text-primary">{formatSeconds(item.totalSeconds)}</span>
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t border-border bg-background/50">
					<td class="px-6 py-3 text-sm font-medium text-text">合計</td>
					<td class="px-6 py-3 text-right text-lg font-bold text-primary">{formatHours(data.totalHours)}</td>
				</tr>
			</tfoot>
		</table>
	</div>
{/if}

<!-- Empty State -->
{#if data.workItemStats.length === 0}
	<div class="rounded-xl border border-border bg-surface p-12 text-center shadow-sm">
		<svg class="mx-auto h-12 w-12 text-text-muted opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
			<rect x="3" y="4" width="18" height="18" rx="2" />
			<path d="M16 2v4M8 2v4M3 10h18" />
		</svg>
		<p class="mt-4 text-text-muted">本月尚無工作紀錄</p>
	</div>
{/if}
