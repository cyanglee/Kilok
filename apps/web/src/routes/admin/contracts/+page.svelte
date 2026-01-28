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
	let editingContract = $state<(typeof data.contracts)[0] | null>(null);
	let deletingContract = $state<(typeof data.contracts)[0] | null>(null);
	let isDeleting = $state(false);

	function openCreateDialog() {
		editingContract = null;
		$form = {
			client_id: data.clients[0]?.id ?? 0,
			year: new Date().getFullYear(),
			total_hours: 0,
			carried_over: 0
		};
		dialogOpen = true;
	}

	function openEditDialog(contract: (typeof data.contracts)[0]) {
		editingContract = contract;
		$form = {
			id: contract.id,
			client_id: contract.client_id,
			year: contract.year,
			total_hours: contract.total_hours,
			carried_over: contract.carried_over
		};
		dialogOpen = true;
	}

	function openDeleteDialog(contract: (typeof data.contracts)[0]) {
		deletingContract = contract;
		deleteDialogOpen = true;
	}

	function formatHours(hours: number): string {
		return hours.toLocaleString('zh-TW');
	}
</script>

<svelte:head>
	<title>合約管理 | Kilok</title>
</svelte:head>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-start justify-between">
		<div class="space-y-1">
			<h1 class="font-heading text-2xl font-bold text-slate-900">合約管理</h1>
			<p class="text-slate-500">管理客戶年度合約時數與結轉</p>
		</div>
		<Button onclick={openCreateDialog} disabled={data.clients.length === 0} class="gap-2">
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
			</svg>
			新增合約
		</Button>
	</div>

	{#if data.clients.length === 0}
		<div class="flex items-center gap-3 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3">
			<svg class="h-5 w-5 flex-shrink-0 text-amber-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
			</svg>
			<span class="text-sm font-medium text-amber-800">請先建立客戶後才能新增合約</span>
		</div>
	{/if}

	<!-- Stats Cards -->
	<div class="grid gap-4 sm:grid-cols-3">
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-indigo-100">
					<svg class="h-6 w-6 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總合約數</p>
					<p class="text-2xl font-bold text-slate-900">{data.contracts.length}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-emerald-100">
					<svg class="h-6 w-6 text-emerald-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總合約時數</p>
					<p class="text-2xl font-bold text-slate-900">{formatHours(data.contracts.reduce((acc, c) => acc + c.total_hours, 0))}</p>
				</div>
			</div>
		</div>
		<div class="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
			<div class="flex items-center gap-4">
				<div class="flex h-12 w-12 items-center justify-center rounded-lg bg-cyan-100">
					<svg class="h-6 w-6 text-cyan-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
						<path stroke-linecap="round" stroke-linejoin="round" d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
					</svg>
				</div>
				<div>
					<p class="text-sm font-medium text-slate-500">總結轉時數</p>
					<p class="text-2xl font-bold text-slate-900">{formatHours(data.contracts.reduce((acc, c) => acc + c.carried_over, 0))}</p>
				</div>
			</div>
		</div>
	</div>

	<!-- Contract List -->
	{#if data.contracts.length === 0}
		<div class="flex flex-col items-center justify-center rounded-2xl border-2 border-dashed border-slate-200 bg-white py-16">
			<div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-slate-100">
				<svg class="h-8 w-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
					<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
				</svg>
			</div>
			<p class="text-lg font-medium text-slate-900">尚無合約</p>
			<p class="mt-1 text-sm text-slate-500">點擊「新增合約」按鈕開始建立</p>
		</div>
	{:else}
		<div class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
			<ul class="divide-y divide-slate-100">
				{#each data.contracts as contract}
					<li class="group relative hover:bg-slate-50 transition-colors">
						<div class="px-6 py-4">
							<div class="flex items-center justify-between gap-4">
								<!-- Left: Avatar + Info -->
								<div class="flex items-center gap-4 min-w-0 flex-1">
									<div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 text-sm font-bold text-white shadow-sm">
										{contract.client_name.charAt(0)}
									</div>
									<div class="min-w-0 flex-1">
										<div class="flex items-center gap-3">
											<h3 class="font-semibold text-slate-900">{contract.client_name}</h3>
											<span class="inline-flex items-center rounded-full bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-600">
												{contract.year}
											</span>
										</div>
									</div>
								</div>

								<!-- Center: Hours -->
								<div class="hidden sm:flex items-center gap-6 text-sm">
									<div class="text-center">
										<p class="text-xs text-slate-400">合約</p>
										<p class="font-mono font-semibold text-slate-700">{formatHours(contract.total_hours)}h</p>
									</div>
									<div class="text-center">
										<p class="text-xs text-slate-400">結轉</p>
										<p class="font-mono font-semibold {contract.carried_over > 0 ? 'text-cyan-600' : 'text-slate-400'}">
											{#if contract.carried_over > 0}+{/if}{formatHours(contract.carried_over)}h
										</p>
									</div>
									<div class="text-center px-3 py-1 rounded-lg bg-primary/10">
										<p class="text-xs text-primary/70">總計</p>
										<p class="font-mono font-bold text-primary">{formatHours(contract.total_hours + contract.carried_over)}h</p>
									</div>
								</div>

								<!-- Right: Actions -->
								<div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
									<button
										onclick={() => openEditDialog(contract)}
										aria-label="編輯合約"
										class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-slate-200 hover:text-slate-600"
									>
										<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125" />
										</svg>
									</button>
									<button
										onclick={() => openDeleteDialog(contract)}
										aria-label="刪除合約"
										class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-colors hover:bg-red-50 hover:text-red-500"
									>
										<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
											<path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 00-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
										</svg>
									</button>
								</div>
							</div>

							<!-- Mobile hours (shown below on small screens) -->
							<div class="mt-3 flex items-center gap-4 text-sm sm:hidden">
								<div>
									<span class="text-slate-400">合約:</span>
									<span class="font-mono font-semibold text-slate-700">{formatHours(contract.total_hours)}h</span>
								</div>
								<div>
									<span class="text-slate-400">結轉:</span>
									<span class="font-mono font-semibold {contract.carried_over > 0 ? 'text-cyan-600' : 'text-slate-400'}">
										{#if contract.carried_over > 0}+{/if}{formatHours(contract.carried_over)}h
									</span>
								</div>
								<div class="ml-auto px-2 py-0.5 rounded bg-primary/10">
									<span class="font-mono font-bold text-primary">{formatHours(contract.total_hours + contract.carried_over)}h</span>
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
			<Dialog.Title>{editingContract ? '編輯合約' : '新增合約'}</Dialog.Title>
			<Dialog.Description>
				{editingContract ? '修改合約資料' : '建立新的年度合約'}
			</Dialog.Description>
		</Dialog.Header>

		<form
			method="POST"
			action={editingContract ? '?/update' : '?/create'}
			use:formEnhance
			class="space-y-4"
		>
			{#if editingContract}
				<input type="hidden" name="id" value={editingContract.id} />
			{/if}

			<div class="space-y-2">
				<Label for="client_id">客戶</Label>
				{#if editingContract}
					<p class="rounded bg-muted px-3 py-2 text-sm">
						{editingContract.client_name}
					</p>
					<input type="hidden" name="client_id" value={editingContract.client_id} />
				{:else}
					<select
						id="client_id"
						name="client_id"
						bind:value={$form.client_id}
						class="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-xs transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
					>
						{#each data.clients as client}
							<option value={client.id}>{client.name}</option>
						{/each}
					</select>
				{/if}
				{#if $errors.client_id}
					<p class="text-sm text-danger">{$errors.client_id}</p>
				{/if}
			</div>

			<div class="space-y-2">
				<Label for="year">年度</Label>
				<Input
					id="year"
					name="year"
					type="number"
					bind:value={$form.year}
					min={2020}
					max={2100}
				/>
				{#if $errors.year}
					<p class="text-sm text-danger">{$errors.year}</p>
				{/if}
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="total_hours">合約時數</Label>
					<Input
						id="total_hours"
						name="total_hours"
						type="number"
						bind:value={$form.total_hours}
						min={0}
						step={0.5}
					/>
					{#if $errors.total_hours}
						<p class="text-sm text-danger">{$errors.total_hours}</p>
					{/if}
				</div>

				<div class="space-y-2">
					<Label for="carried_over">結轉時數</Label>
					<Input
						id="carried_over"
						name="carried_over"
						type="number"
						bind:value={$form.carried_over}
						min={0}
						step={0.5}
					/>
					{#if $errors.carried_over}
						<p class="text-sm text-danger">{$errors.carried_over}</p>
					{/if}
					<p class="text-xs text-text-muted">
						從上一年度結轉的剩餘時數
					</p>
				</div>
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
				<Button type="submit">
					{editingContract ? '儲存' : '建立'}
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
				確定要刪除「{deletingContract?.client_name}」{deletingContract?.year} 年的合約嗎？此操作無法復原。
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
			<input type="hidden" name="id" value={deletingContract?.id} />

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
