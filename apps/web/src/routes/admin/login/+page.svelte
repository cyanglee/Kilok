<script lang="ts">
	import { enhance } from '$app/forms';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';

	let { form } = $props();
	let loading = $state(false);
</script>

<svelte:head>
	<title>管理員登入 | Kilok</title>
</svelte:head>

<div class="flex min-h-screen">
	<!-- Left Panel - Branding -->
	<div class="hidden w-1/2 bg-gradient-to-br from-primary via-blue-600 to-blue-800 lg:flex lg:flex-col lg:justify-between p-12">
		<div>
			<div class="flex items-center gap-3">
				<div class="flex h-12 w-12 items-center justify-center rounded-xl bg-white/20 backdrop-blur-sm">
					<svg class="h-7 w-7 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<span class="font-heading text-2xl font-bold text-white">Kilok</span>
			</div>
		</div>

		<div class="space-y-6">
			<h1 class="font-heading text-4xl font-bold leading-tight text-white">
				工時追蹤<br />簡單高效
			</h1>
			<p class="text-lg text-blue-100">
				管理客戶、專案與合約，<br />
				輕鬆掌握每一分工作時間。
			</p>
		</div>

		<div class="flex items-center gap-4 text-sm text-blue-200">
			<span>© 2024 Kilok</span>
			<span>·</span>
			<a href="/" class="hover:text-white transition-colors">返回首頁</a>
		</div>
	</div>

	<!-- Right Panel - Login Form -->
	<div class="flex w-full items-center justify-center bg-slate-50 p-8 lg:w-1/2">
		<div class="w-full max-w-md space-y-8">
			<!-- Mobile Logo -->
			<div class="flex items-center gap-3 lg:hidden">
				<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary">
					<svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
					</svg>
				</div>
				<span class="font-heading text-xl font-bold text-slate-900">Kilok</span>
			</div>

			<!-- Form Header -->
			<div class="space-y-2">
				<h2 class="font-heading text-3xl font-bold text-slate-900">歡迎回來</h2>
				<p class="text-slate-600">請輸入管理密碼以存取後台</p>
			</div>

			<!-- Login Form -->
			<form
				method="POST"
				use:enhance={() => {
					loading = true;
					return async ({ update }) => {
						await update();
						loading = false;
					};
				}}
				class="space-y-6"
			>
				<div class="space-y-2">
					<Label for="password" class="text-sm font-medium text-slate-700">管理密碼</Label>
					<div class="relative">
						<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
							<svg class="h-5 w-5 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z" />
							</svg>
						</div>
						<Input
							id="password"
							name="password"
							type="password"
							placeholder="輸入密碼"
							autocomplete="current-password"
							disabled={loading}
							required
							class="h-12 pl-10 text-base"
						/>
					</div>
				</div>

				{#if form?.error}
					<div class="flex items-center gap-2 rounded-lg bg-red-50 px-4 py-3 text-sm text-red-600">
						<svg class="h-5 w-5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
						</svg>
						{form.error}
					</div>
				{/if}

				<Button type="submit" class="h-12 w-full text-base font-semibold" disabled={loading}>
					{#if loading}
						<svg class="mr-2 h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						登入中...
					{:else}
						登入後台
						<svg class="ml-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
						</svg>
					{/if}
				</Button>
			</form>

			<!-- Footer -->
			<p class="text-center text-sm text-slate-500 lg:hidden">
				<a href="/" class="text-primary hover:underline">返回首頁</a>
			</p>
		</div>
	</div>
</div>
