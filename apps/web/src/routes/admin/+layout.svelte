<script lang="ts">
	import { page } from '$app/state';
	import { cn } from '$lib/utils';

	let { children, data } = $props();

	const navItems = [
		{ href: '/admin/clients', label: '客戶', icon: 'users' },
		{ href: '/admin/projects', label: '專案', icon: 'folder' },
		{ href: '/admin/contracts', label: '合約', icon: 'file-text' }
	];
</script>

{#if !data.authenticated}
	{@render children()}
{:else}
	<div class="min-h-screen bg-slate-50">
		<!-- Sidebar -->
		<aside class="fixed inset-y-0 left-0 z-50 w-64 border-r border-slate-200 bg-white">
			<!-- Logo -->
			<div class="flex h-16 items-center gap-3 border-b border-slate-200 px-6">
				<div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary">
					<svg class="h-5 w-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<div>
					<a href="/" class="font-heading text-lg font-bold text-slate-900">Kilok</a>
					<p class="text-xs text-slate-500">管理後台</p>
				</div>
			</div>

			<!-- Navigation -->
			<nav class="flex flex-col gap-1 p-4">
				<p class="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-slate-400">管理</p>
				{#each navItems as item}
					<a
						href={item.href}
						class={cn(
							'flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all duration-200',
							page.url.pathname.startsWith(item.href)
								? 'bg-primary/10 text-primary shadow-sm'
								: 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
						)}
					>
						{#if item.icon === 'users'}
							<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
							</svg>
						{:else if item.icon === 'folder'}
							<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
							</svg>
						{:else if item.icon === 'file-text'}
							<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
							</svg>
						{/if}
						{item.label}
					</a>
				{/each}
			</nav>

			<!-- Bottom -->
			<div class="absolute bottom-0 left-0 right-0 border-t border-slate-200 p-4">
				<form method="POST" action="/admin/logout">
					<button
						type="submit"
						class="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium text-slate-600 transition-colors hover:bg-red-50 hover:text-red-600"
					>
						<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
							<path stroke-linecap="round" stroke-linejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15m3 0l3-3m0 0l-3-3m3 3H9" />
						</svg>
						登出
					</button>
				</form>
			</div>
		</aside>

		<!-- Main Content -->
		<main class="ml-64 min-h-screen">
			<div class="p-8">
				{@render children()}
			</div>
		</main>
	</div>
{/if}
