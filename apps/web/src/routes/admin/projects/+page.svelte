<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { superForm } from 'sveltekit-superforms';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';

	let { data } = $props();

	// Form handling
	const { form, errors, message, enhance: formEnhance } = superForm(data.form, {
		resetForm: false,
		onResult: ({ result }) => {
			if (result.type === 'success') {
				dialogOpen = false;
				invalidateAll();
			}
		}
	});

	// Dialog state
	let dialogOpen = $state(false);
	let editingProject = $state<(typeof data.projects)[0] | null>(null);
	let workItemExample = $state('');
	let isTogglingIgnore = $state(false);

	function openEditDialog(project: (typeof data.projects)[0]) {
		editingProject = project;
		$form = {
			id: project.id,
			display_name: project.display_name,
			client_id: project.client_id,
			work_item_pattern: project.work_item_pattern
		};
		workItemExample = '';
		dialogOpen = true;
	}

	function inferPattern(example: string): string {
		if (!example.trim()) return '';

		// ABC-123 (Jira/Linear style)
		const match1 = example.match(/^([A-Z]+)-(\d+)$/i);
		if (match1) return `${match1[1].toUpperCase()}-\\d+`;

		// Pure number: 123
		if (/^\d+$/.test(example)) return '\\d+';

		// ABC123 (no separator)
		const match2 = example.match(/^([A-Z]+)(\d+)$/i);
		if (match2) return `${match2[1].toUpperCase()}\\d+`;

		// 123-description (GitHub issue style)
		const match3 = example.match(/^(\d+)-(.+)$/);
		if (match3) return '\\d+-.+';

		// #123 (GitHub PR/issue reference)
		const match4 = example.match(/^#(\d+)$/);
		if (match4) return '#\\d+';

		// feature/ABC-123 or fix/ABC-123 (branch with prefix)
		const match5 = example.match(/^(feat|fix|feature|bugfix|hotfix|chore|docs|refactor)\/([A-Z]+)-(\d+)/i);
		if (match5) return `${match5[1].toLowerCase()}/[A-Z]+-\\d+`;

		// feature/123-description (branch with number prefix)
		const match6 = example.match(/^(feat|fix|feature|bugfix|hotfix|chore|docs|refactor)\/(\d+)-/i);
		if (match6) return `${match6[1].toLowerCase()}/\\d+-.+`;

		return example;
	}

	function handleExampleInput() {
		if (workItemExample) {
			$form.work_item_pattern = inferPattern(workItemExample);
		}
	}

	// Get client name by ID
	function getClientName(clientId: number | null): string {
		if (!clientId) return '未指定';
		const client = data.clients.find((c) => c.id === clientId);
		return client?.name ?? '未知';
	}

	// Shorten path by replacing home directory with ~
	function shortenPath(path: string): string {
		return path.replace(/^\/Users\/[^/]+/, '~');
	}
</script>

<svelte:head>
	<title>專案管理 | Kilok</title>
</svelte:head>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div class="space-y-1">
			<h1 class="font-heading text-2xl font-bold text-slate-900">專案管理</h1>
			<p class="text-slate-500">管理由 CLI 自動建立的專案，設定顯示名稱與 Work Item 模式</p>
		</div>
		<a
			href={data.showIgnored ? '/admin/projects' : '/admin/projects?showIgnored=true'}
			class="inline-flex items-center gap-2 rounded-lg border border-slate-200 px-3 py-2 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-50"
		>
			{#if data.showIgnored}
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
				</svg>
				隱藏已忽略
			{:else}
				<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
					<path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
				</svg>
				顯示已忽略
			{/if}
		</a>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 sm:grid-cols-3">
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-violet-100">
					<svg class="h-6 w-6 text-violet-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總專案數</p>
					<p class="text-2xl font-bold text-slate-900">{data.projects.length}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-100">
					<svg class="h-6 w-6 text-emerald-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m13.35-.622l1.757-1.757a4.5 4.5 0 00-6.364-6.364l-4.5 4.5a4.5 4.5 0 001.242 7.244" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">已連結客戶</p>
					<p class="text-2xl font-bold text-slate-900">{data.projects.filter(p => p.client_id).length}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-amber-100">
					<svg class="h-6 w-6 text-amber-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M5.25 8.25h15m-16.5 7.5h15m-1.8-13.5l-3.9 19.5m-2.1-19.5l-3.9 19.5" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">有 Work Item</p>
					<p class="text-2xl font-bold text-slate-900">{data.projects.filter(p => p.work_item_pattern).length}</p>
				</div>
			</div>
		</div>
	</div>

	<!-- Project List -->
	{#if data.projects.length === 0}
		<div class="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-slate-200 bg-white py-16">
			<div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-slate-100">
				<svg class="h-8 w-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
					<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
				</svg>
			</div>
			<p class="text-lg font-medium text-slate-900">尚無專案</p>
			<p class="mt-1 text-sm text-slate-500">專案會由 CLI 工具自動建立</p>
		</div>
	{:else}
		<div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
			<ul class="divide-y divide-slate-100">
				{#each data.projects as project}
					<li class="group relative hover:bg-slate-50 transition-colors">
						<div class="px-6 py-4">
							<!-- Main row -->
							<div class="flex items-start justify-between gap-4">
								<!-- Left: Icon + Info -->
								<div class="flex items-start gap-4 min-w-0 flex-1">
									<div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-purple-600 shadow-sm">
										<svg class="h-5 w-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
										</svg>
									</div>
									<div class="min-w-0 flex-1">
										<h3 class="font-semibold text-slate-900">
											{project.display_name || '未命名'}
										</h3>
										<code class="mt-1 block text-sm text-slate-500 font-mono truncate" title={project.path}>
											{shortenPath(project.path)}
										</code>
									</div>
								</div>

								<!-- Right: Badges + Actions -->
								<div class="flex items-center gap-3 flex-shrink-0">
									<!-- Badges -->
									<div class="hidden sm:flex items-center gap-2">
										{#if project.ignored}
											<span class="inline-flex items-center gap-1.5 rounded-full bg-slate-200 px-2.5 py-1 text-xs font-medium text-slate-500">
												<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
													<path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
												</svg>
												已忽略
											</span>
										{:else if project.client_id}
											<span class="inline-flex items-center gap-1.5 rounded-full bg-blue-100 px-2.5 py-1 text-xs font-medium text-blue-700">
												<svg class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
													<path stroke-linecap="round" stroke-linejoin="round" d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.501 20.118a7.5 7.5 0 0114.998 0A17.933 17.933 0 0112 21.75c-2.676 0-5.216-.584-7.499-1.632z" />
												</svg>
												{getClientName(project.client_id)}
											</span>
										{:else}
											<span class="inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500">
												未指定客戶
											</span>
										{/if}
									</div>

									<!-- Actions -->
									<div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
										<button
											onclick={() => openEditDialog(project)}
											aria-label="編輯專案"
											class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-slate-200 hover:text-slate-600"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
												<path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125" />
											</svg>
										</button>
										<form
											method="POST"
											action="?/toggleIgnore"
											use:enhance={() => {
												isTogglingIgnore = true;
												return async ({ update }) => {
													await update();
													isTogglingIgnore = false;
													invalidateAll();
												};
											}}
										>
											<input type="hidden" name="id" value={project.id} />
											<input type="hidden" name="ignored" value={project.ignored ? 'false' : 'true'} />
											<button
												type="submit"
												disabled={isTogglingIgnore}
												aria-label={project.ignored ? '取消忽略' : '忽略專案'}
												title={project.ignored ? '取消忽略' : '忽略專案'}
												class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors {project.ignored ? 'hover:bg-emerald-50 hover:text-emerald-500' : 'hover:bg-amber-50 hover:text-amber-500'}"
											>
												{#if project.ignored}
													<!-- Eye icon to show/restore -->
													<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
														<path stroke-linecap="round" stroke-linejoin="round" d="M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z" />
														<path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
													</svg>
												{:else}
													<!-- Eye-slash icon to hide -->
													<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
														<path stroke-linecap="round" stroke-linejoin="round" d="M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88" />
													</svg>
												{/if}
											</button>
										</form>
									</div>
								</div>
							</div>

							<!-- Mobile badges (shown below on small screens) -->
							<div class="mt-3 flex flex-wrap items-center gap-2 sm:hidden">
								{#if project.ignored}
									<span class="inline-flex items-center gap-1.5 rounded-full bg-slate-200 px-2.5 py-1 text-xs font-medium text-slate-500">
										已忽略
									</span>
								{:else if project.client_id}
									<span class="inline-flex items-center gap-1.5 rounded-full bg-blue-100 px-2.5 py-1 text-xs font-medium text-blue-700">
										{getClientName(project.client_id)}
									</span>
								{:else}
									<span class="inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500">
										未指定客戶
									</span>
								{/if}
							</div>
						</div>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>

<!-- Edit Dialog -->
<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>編輯專案</Dialog.Title>
			<Dialog.Description>
				修改專案設定。路徑由系統自動偵測，無法手動更改。
			</Dialog.Description>
		</Dialog.Header>

		<form method="POST" action="?/update" use:formEnhance class="space-y-4">
			<input type="hidden" name="id" value={$form.id} />

			<div class="space-y-2">
				<Label>專案路徑</Label>
				<p class="rounded bg-muted px-3 py-2 font-mono text-sm text-text-muted">
					{editingProject?.path}
				</p>
			</div>

			<div class="space-y-2">
				<Label for="display_name">顯示名稱</Label>
				<Input
					id="display_name"
					name="display_name"
					bind:value={$form.display_name}
					placeholder="例如：公司官網"
				/>
				{#if $errors.display_name}
					<p class="text-sm text-danger">{$errors.display_name}</p>
				{/if}
			</div>

			<div class="space-y-2">
				<Label for="client_id">所屬客戶</Label>
				<select
					id="client_id"
					name="client_id"
					bind:value={$form.client_id}
					class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-xs transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
				>
					<option value={null}>未指定</option>
					{#each data.clients as client}
						<option value={client.id}>{client.name}</option>
					{/each}
				</select>
			</div>

			<div class="space-y-2">
				<Label for="work_item_example">Work Item 範例</Label>
				<Input
					id="work_item_example"
					name="work_item_example"
					bind:value={workItemExample}
					oninput={handleExampleInput}
					placeholder="例如：BRU-321"
				/>
				<p class="text-xs text-text-muted">
					輸入一個範例，系統會自動推導正則表達式
				</p>
			</div>

			<div class="space-y-2">
				<Label for="work_item_pattern">Work Item 模式（Regex）</Label>
				<Input
					id="work_item_pattern"
					name="work_item_pattern"
					bind:value={$form.work_item_pattern}
					placeholder="例如：BRU-\d+"
					class="font-mono"
				/>
				{#if $errors.work_item_pattern}
					<p class="text-sm text-danger">{$errors.work_item_pattern}</p>
				{/if}
			</div>

			{#if $message}
				<p class={$message.type === 'error' ? 'text-sm text-danger' : 'text-sm text-success'}>
					{$message.text}
				</p>
			{/if}

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (dialogOpen = false)}>
					取消
				</Button>
				<Button type="submit">儲存</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

