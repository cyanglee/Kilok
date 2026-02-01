<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { superForm } from 'sveltekit-superforms';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Dialog from '$lib/components/ui/dialog';
	import { generateSlug } from '$lib/slug';

	let { data } = $props();

	// Form handling
	const { form, errors, message, enhance: formEnhance } = superForm(data.form, {
		resetForm: true,
		onResult: ({ result }) => {
			if (result.type === 'success') {
				dialogOpen = false;
				invalidateAll();
			}
		}
	});

	// Dialog state
	let dialogOpen = $state(false);
	let deleteDialogOpen = $state(false);
	let editingClient = $state<typeof data.clients[0] | null>(null);
	let deletingClient = $state<typeof data.clients[0] | null>(null);
	let isDeleting = $state(false);
	let copiedToken = $state<string | null>(null);

	function copyShareLink(token: string) {
		const url = `${window.location.origin}/share/${token}`;
		navigator.clipboard.writeText(url);
		copiedToken = token;
		setTimeout(() => {
			copiedToken = null;
		}, 2000);
	}

	function openCreateDialog() {
		editingClient = null;
		$form = { name: '', slug: '' };
		dialogOpen = true;
	}

	function openEditDialog(client: typeof data.clients[0]) {
		editingClient = client;
		$form = { id: client.id, name: client.name, slug: client.slug };
		dialogOpen = true;
	}

	function openDeleteDialog(client: typeof data.clients[0]) {
		deletingClient = client;
		deleteDialogOpen = true;
	}

	function handleNameInput() {
		// Only auto-generate slug for new clients
		if (!editingClient) {
			$form.slug = generateSlug($form.name);
		}
	}
</script>

<svelte:head>
	<title>客戶管理 | Kilok</title>
</svelte:head>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div class="space-y-1">
			<h1 class="font-heading text-2xl font-bold text-slate-900">客戶管理</h1>
			<p class="text-slate-500">管理所有客戶資料與相關專案</p>
		</div>
		<Button onclick={openCreateDialog} class="gap-2">
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
			</svg>
			新增客戶
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 sm:grid-cols-3">
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-blue-100">
					<svg class="h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總客戶數</p>
					<p class="text-2xl font-bold text-slate-900">{data.clients.length}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-100">
					<svg class="h-6 w-6 text-emerald-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總專案數</p>
					<p class="text-2xl font-bold text-slate-900">{data.clients.reduce((acc, c) => acc + c.projectCount, 0)}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-amber-100">
					<svg class="h-6 w-6 text-amber-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">活躍客戶</p>
					<p class="text-2xl font-bold text-slate-900">{data.clients.filter(c => c.projectCount > 0).length}</p>
				</div>
			</div>
		</div>
	</div>

	<!-- Client List -->
	{#if data.clients.length === 0}
		<div class="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-slate-200 bg-white py-16">
			<div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-slate-100">
				<svg class="h-8 w-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
					<path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
				</svg>
			</div>
			<p class="text-lg font-medium text-slate-900">尚無客戶</p>
			<p class="mt-1 text-sm text-slate-500">點擊「新增客戶」按鈕開始建立</p>
		</div>
	{:else}
		<div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
			<ul class="divide-y divide-slate-100">
				{#each data.clients as client}
					<li class="group relative hover:bg-slate-50 transition-colors">
						<div class="px-6 py-4">
							<div class="flex items-center justify-between gap-4">
								<!-- Left: Avatar + Info -->
								<div class="flex items-center gap-4 min-w-0 flex-1">
									<div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-primary to-blue-600 text-sm font-bold text-white shadow-sm">
										{client.name.charAt(0)}
									</div>
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-3">
											<h3 class="font-semibold text-slate-900">{client.name}</h3>
											<code class="text-xs text-slate-400 font-mono">/{client.slug}</code>
										</div>
										<div class="mt-0.5 flex items-center gap-3 text-sm text-slate-500">
											<span>{client.projectCount} 個專案</span>
											{#if client.share_token}
												<button
													onclick={() => copyShareLink(client.share_token!)}
													class="inline-flex items-center gap-1 text-primary hover:text-primary/80 transition-colors"
												>
													<svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
														<path stroke-linecap="round" stroke-linejoin="round" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
													</svg>
													{copiedToken === client.share_token ? '已複製!' : '複製分享連結'}
												</button>
											{/if}
										</div>
									</div>
								</div>

								<!-- Right: Status + Actions -->
								<div class="flex items-center gap-3 flex-shrink-0">
									{#if client.projectCount > 0}
										<span class="hidden sm:inline-flex items-center gap-1.5 rounded-full bg-emerald-50 px-2.5 py-1 text-xs font-medium text-emerald-700">
											<span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
											活躍中
										</span>
									{:else}
										<span class="hidden sm:inline-flex items-center gap-1.5 rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-500">
											<span class="h-1.5 w-1.5 rounded-full bg-slate-400"></span>
											待啟用
										</span>
									{/if}

									<!-- Actions -->
									<div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
										<button
											onclick={() => openEditDialog(client)}
											aria-label="編輯客戶"
											class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-slate-200 hover:text-slate-600"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
												<path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125" />
											</svg>
										</button>
										<button
											onclick={() => openDeleteDialog(client)}
											aria-label="刪除客戶"
											disabled={client.projectCount > 0}
											class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-red-50 hover:text-red-500 disabled:cursor-not-allowed disabled:opacity-30"
										>
											<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
												<path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
											</svg>
										</button>
									</div>
								</div>
							</div>
						</div>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>

<!-- Create/Edit Dialog -->
<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>{editingClient ? '編輯客戶' : '新增客戶'}</Dialog.Title>
			<Dialog.Description>
				{editingClient ? '修改客戶資料' : '建立新的客戶'}
			</Dialog.Description>
		</Dialog.Header>

		<form
			method="POST"
			action={editingClient ? '?/update' : '?/create'}
			use:formEnhance
			class="space-y-4"
		>
			{#if editingClient}
				<input type="hidden" name="id" value={editingClient.id} />
			{/if}

			<div class="space-y-2">
				<Label for="name">客戶名稱</Label>
				<Input
					id="name"
					name="name"
					bind:value={$form.name}
					oninput={handleNameInput}
					placeholder="例如：台灣積體電路"
				/>
				{#if $errors.name}
					<p class="text-sm text-danger">{$errors.name}</p>
				{/if}
			</div>

			<div class="space-y-2">
				<Label for="slug">Slug（網址識別碼）</Label>
				<Input
					id="slug"
					name="slug"
					bind:value={$form.slug}
					placeholder="例如：tsmc"
					class="font-mono"
				/>
				{#if $errors.slug}
					<p class="text-sm text-danger">{$errors.slug}</p>
				{/if}
				<p class="text-xs text-text-muted">用於網址，例如：/tsmc/2024-01</p>
			</div>

			{#if $message}
				<p
					class={$message.type === 'error' ? 'text-sm text-danger' : 'text-sm text-success'}
				>
					{$message.text}
				</p>
			{/if}

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (dialogOpen = false)}>
					取消
				</Button>
				<Button type="submit">
					{editingClient ? '儲存' : '建立'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Confirmation Dialog -->
<Dialog.Root bind:open={deleteDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>確認刪除</Dialog.Title>
			<Dialog.Description>
				確定要刪除客戶「{deletingClient?.name}」嗎？此操作無法復原。
			</Dialog.Description>
		</Dialog.Header>

		<form
			method="POST"
			action="?/delete"
			use:enhance={() => {
				isDeleting = true;
				return async ({ update }) => {
					await update();
					isDeleting = false;
					deleteDialogOpen = false;
					invalidateAll();
				};
			}}
		>
			<input type="hidden" name="id" value={deletingClient?.id} />

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (deleteDialogOpen = false)}>
					取消
				</Button>
				<Button type="submit" variant="destructive" disabled={isDeleting}>
					{isDeleting ? '刪除中...' : '確認刪除'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
