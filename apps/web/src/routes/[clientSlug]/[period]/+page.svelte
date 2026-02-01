<script lang="ts">
	import type { PageData } from './$types';
	import { enhance } from '$app/forms';
	import { formatHours, formatBillableHours, formatDate, monthNames } from '$lib/utils/formatters';
	import { StatsCard, PeriodNavigation, EmptyState } from '$lib/components';

	let { data }: { data: PageData } = $props();

	function formatSeconds(seconds: number): string {
		return formatHours(seconds / 3600);
	}

	// Edit state for work items
	let editingWorkItemId: number | null = $state(null);
	let editTitle = $state('');
	let editDescription = $state('');
	let editCompletedDate = $state('');
	let editBillableHours = $state('');
	let isSaving = $state(false);

	// State for creating new work item
	let showAddForm = $state(false);
	let newProjectId = $state('');
	let newTitle = $state('');
	let newDescription = $state('');
	let newCompletedDate = $state('');
	let newBillableHours = $state('');
	let isCreating = $state(false);

	function startEditing(item: (typeof data.completedWorkItems)[0] | (typeof data.trackingWorkItems)[0]) {
		if (!item.workItemId) return;
		editingWorkItemId = item.workItemId;
		editTitle = item.title ?? '';
		editDescription = item.description ?? '';
		editCompletedDate = item.completedDate ?? '';
		// Use billableHoursOverride if set, otherwise show the calculated value
		editBillableHours = item.billableHoursOverride?.toString() ?? item.billableHours.toString();
	}

	function resetAddForm() {
		showAddForm = false;
		newProjectId = data.projects[0]?.id?.toString() ?? '';
		newTitle = '';
		newDescription = '';
		newCompletedDate = '';
		newBillableHours = '';
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
		editCompletedDate = '';
		editBillableHours = '';
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

<!-- Page Header with Month Navigation -->
<div class="mb-8">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-2xl font-bold text-text">
				{data.year} 年 {monthNames[data.month - 1]} 月報
			</h1>
			<p class="mt-1 text-text-muted">{data.client.name}</p>
		</div>
		<PeriodNavigation
			prevHref="/{data.client.slug}/{data.prevPeriod}"
			nextHref="/{data.client.slug}/{data.nextPeriod}"
		/>
	</div>
</div>

<!-- Stats Cards -->
{@const monthlyRemaining = data.monthlyHours - data.completedBillableHours}
{@const monthlySubtitle = monthlyRemaining >= 0
	? `剩餘 ${formatHours(monthlyRemaining)}`
	: `超額 ${formatHours(-monthlyRemaining)}`}
<div class="mb-8 flex flex-wrap gap-4">
	<StatsCard
		label="本月計費工時"
		value={formatBillableHours(data.completedBillableHours)}
		subtitle="原始 {formatHours(data.totalHours)}"
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

<!-- Completed Work Items Table - 已完成報告 -->
<div class="mb-8 overflow-hidden rounded-xl border border-border bg-surface shadow-sm">
	<div class="border-b border-border bg-background/50 px-6 py-3 flex items-center justify-between">
		<h2 class="text-sm font-semibold text-text">已完成報告</h2>
		<button
			type="button"
			onclick={() => {
				showAddForm = !showAddForm;
				if (showAddForm) {
					newProjectId = data.projects[0]?.id?.toString() ?? '';
					// Default to today's date
					const today = new Date().toISOString().slice(0, 10);
					newCompletedDate = today;
				}
			}}
			class="text-xs text-primary hover:text-primary/80 transition-colors"
		>
			{showAddForm ? '取消' : '+ 新增工作項目'}
		</button>
	</div>

	<!-- Add Work Item Form -->
	{#if showAddForm}
		<form
			method="POST"
			action="?/createWorkItem"
			use:enhance={() => {
				isCreating = true;
				return async ({ result, update }) => {
					isCreating = false;
					if (result.type === 'success') {
						resetAddForm();
					}
					await update();
				};
			}}
			class="border-b border-border bg-background/20 px-6 py-4 space-y-3"
		>
			<div class="grid grid-cols-3 gap-3">
				<div>
					<label for="new-project" class="block text-xs text-text-muted mb-1">專案</label>
					<select
						id="new-project"
						name="project_id"
						bind:value={newProjectId}
						class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text focus:border-primary focus:outline-none"
					>
						{#each data.projects as project}
							<option value={project.id}>{project.display_name ?? project.path}</option>
						{/each}
					</select>
				</div>
				<div>
					<label for="new-completed-date" class="block text-xs text-text-muted mb-1">完成日期</label>
					<input
						id="new-completed-date"
						type="date"
						name="completed_date"
						bind:value={newCompletedDate}
						class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text focus:border-primary focus:outline-none"
					/>
				</div>
				<div>
					<label for="new-billable-hours" class="block text-xs text-text-muted mb-1">計費工時</label>
					<input
						id="new-billable-hours"
						type="number"
						name="billable_hours"
						step="0.5"
						min="0"
						bind:value={newBillableHours}
						placeholder="例如：3.5"
						class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text focus:border-primary focus:outline-none"
					/>
				</div>
			</div>
			<div>
				<label for="new-title" class="block text-xs text-text-muted mb-1">工作項目名稱</label>
				<input
					id="new-title"
					type="text"
					name="title"
					placeholder="例如：頁面文字優化"
					bind:value={newTitle}
					class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text placeholder:text-text-muted focus:border-primary focus:outline-none"
				/>
			</div>
			<div>
				<label for="new-description" class="block text-xs text-text-muted mb-1">工作內容說明（選填）</label>
				<textarea
					id="new-description"
					name="description"
					placeholder="工作內容詳細說明"
					bind:value={newDescription}
					rows="2"
					class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text placeholder:text-text-muted focus:border-primary focus:outline-none"
				></textarea>
			</div>
			<div class="flex gap-2">
				<button
					type="submit"
					disabled={isCreating || !newTitle.trim()}
					class="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary/90 disabled:opacity-50"
				>
					{isCreating ? '建立中...' : '建立工作項目'}
				</button>
				<button
					type="button"
					onclick={resetAddForm}
					class="rounded-lg border border-border px-4 py-2 text-sm font-medium text-text-muted transition-colors hover:bg-border"
				>
					取消
				</button>
			</div>
		</form>
	{/if}

	{#if data.completedWorkItems.length > 0}
		<table class="w-full">
			<thead>
				<tr class="border-b border-border bg-background/30">
					<th class="px-6 py-3 text-left text-xs font-medium text-text-muted w-24 whitespace-nowrap">完成日期</th>
					<th class="px-6 py-3 text-left text-xs font-medium text-text-muted">工作項目</th>
					<th class="px-6 py-3 text-right text-xs font-medium text-text-muted w-24">計費工時</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-border">
				{#each data.completedWorkItems as item}
					<tr class="group hover:bg-background/30 transition-colors">
						<td class="px-6 py-4 text-sm text-text-muted font-mono">
							{item.completedDate ? formatDate(item.completedDate) : '-'}
						</td>
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
									class="space-y-3"
								>
									<input type="hidden" name="id" value={item.workItemId} />
									<div class="grid grid-cols-2 gap-3">
										<div>
											<label for="edit-completed-date" class="block text-xs text-text-muted mb-1">完成日期</label>
											<input
												id="edit-completed-date"
												type="date"
												name="completed_date"
												bind:value={editCompletedDate}
												class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text focus:border-primary focus:outline-none"
											/>
										</div>
										<div>
											<label for="edit-billable-hours" class="block text-xs text-text-muted mb-1">計費工時</label>
											<input
												id="edit-billable-hours"
												type="number"
												name="billable_hours"
												step="0.5"
												min="0"
												bind:value={editBillableHours}
												placeholder="例如：3.5"
												class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm text-text focus:border-primary focus:outline-none"
											/>
										</div>
									</div>
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
	{:else if !showAddForm}
		<!-- Empty state when no completed items and not adding -->
		<div class="px-6 py-8 text-center text-text-muted text-sm">
			尚無已完成報告。點擊「+ 新增工作項目」來建立第一個。
		</div>
	{/if}
</div>

<!-- Tracking Work Items Table - 追蹤中 (Admin only) -->
{#if data.trackingWorkItems.length > 0}
	<div class="overflow-hidden rounded-xl border border-border/50 bg-surface/50 shadow-sm">
		<div class="border-b border-border/50 bg-background/30 px-6 py-3 flex items-center gap-2">
			<span class="text-amber-500">
				<svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="12" cy="12" r="10" />
					<polyline points="12 6 12 12 16 14" />
				</svg>
			</span>
			<h2 class="text-sm font-semibold text-text-muted">追蹤中</h2>
			<span class="text-xs text-text-muted">(尚未產生報告)</span>
		</div>
		<table class="w-full">
			<thead>
				<tr class="border-b border-border/50 bg-background/20">
					<th class="px-6 py-3 text-left text-xs font-medium text-text-muted">Branch</th>
					<th class="px-6 py-3 text-right text-xs font-medium text-text-muted w-24">原始時間</th>
					<th class="px-6 py-3 text-right text-xs font-medium text-text-muted w-20">Sessions</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-border/50">
				{#each data.trackingWorkItems as item}
					<tr class="group hover:bg-background/20 transition-colors">
						<td class="px-6 py-3">
							<div class="flex items-center gap-2">
								<code class="text-sm text-text-muted font-mono">{item.branch}</code>
								{#if item.workItemId}
									<button
										onclick={() => startEditing(item)}
										class="opacity-0 group-hover:opacity-100 transition-opacity text-xs text-primary hover:text-primary/80"
									>
										編輯
									</button>
								{/if}
							</div>
							{#if item.title}
								<div class="text-xs text-text-muted mt-1">{item.title}</div>
							{/if}
						</td>
						<td class="px-6 py-3 text-right whitespace-nowrap">
							<span class="text-sm text-text-muted">{formatSeconds(item.totalSeconds)}</span>
						</td>
						<td class="px-6 py-3 text-right whitespace-nowrap">
							<span class="text-sm text-text-muted">{item.sessionCount}</span>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<!-- Empty State (only show when no tracking items and no form open) -->
{#if data.trackingWorkItems.length === 0 && data.completedWorkItems.length === 0 && !showAddForm}
	<EmptyState message="本月尚無工作紀錄" />
{/if}
