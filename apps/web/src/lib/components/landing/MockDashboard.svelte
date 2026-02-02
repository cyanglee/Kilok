<script lang="ts">
	// Mock data for demonstration
	const mockStats = {
		monthlyHours: 12.5,
		yearlyHours: 89.0,
		projectCount: 3
	};

	const mockClients = [
		{
			name: 'Acme Corp',
			monthlyHours: 8.5,
			yearlyHours: 62.0,
			contractHours: 96,
			projectCount: 2
		},
		{
			name: 'Startup Inc',
			monthlyHours: 3.5,
			yearlyHours: 24.0,
			contractHours: 48,
			projectCount: 1
		}
	];

	function formatHours(hours: number): string {
		return hours.toFixed(1) + 'h';
	}
</script>

<section class="bg-slate-50 px-4 py-20">
	<div class="mx-auto max-w-5xl">
		<!-- Section header -->
		<div class="mb-12 text-center">
			<h2 class="font-heading text-3xl font-bold text-text sm:text-4xl">
				客戶報告一目瞭然
			</h2>
			<p class="mt-4 text-lg text-text-muted">
				用 Web 儀表板管理所有客戶與專案
			</p>
		</div>

		<!-- Mock Dashboard -->
		<div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl">
			<!-- Browser chrome -->
			<div class="flex items-center gap-2 border-b border-slate-200 bg-slate-100 px-4 py-3">
				<div class="flex gap-1.5">
					<div class="h-3 w-3 rounded-full bg-red-400"></div>
					<div class="h-3 w-3 rounded-full bg-yellow-400"></div>
					<div class="h-3 w-3 rounded-full bg-green-400"></div>
				</div>
				<div class="flex-1 text-center">
					<div class="mx-auto inline-flex items-center gap-2 rounded-md bg-white px-3 py-1 text-xs text-slate-500">
						<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
						</svg>
						kilok.com/dashboard
					</div>
				</div>
			</div>

			<!-- Dashboard content -->
			<div class="p-6">
				<!-- Header -->
				<div class="mb-6">
					<h3 class="text-2xl font-bold text-slate-900">工時總覽</h3>
					<p class="text-sm text-slate-500">2026 年 2 月</p>
				</div>

				<!-- Stats cards -->
				<div class="mb-6 grid grid-cols-3 gap-4">
					<div class="rounded-xl border border-slate-200 bg-slate-50 p-4">
						<div class="text-xs font-medium text-slate-500">本月工時</div>
						<div class="mt-1 text-2xl font-bold text-primary">{formatHours(mockStats.monthlyHours)}</div>
					</div>
					<div class="rounded-xl border border-slate-200 bg-slate-50 p-4">
						<div class="text-xs font-medium text-slate-500">年度累計</div>
						<div class="mt-1 text-2xl font-bold text-slate-900">{formatHours(mockStats.yearlyHours)}</div>
					</div>
					<div class="rounded-xl border border-slate-200 bg-slate-50 p-4">
						<div class="text-xs font-medium text-slate-500">追蹤專案</div>
						<div class="mt-1 text-2xl font-bold text-secondary">{mockStats.projectCount}</div>
					</div>
				</div>

				<!-- Client cards -->
				<div class="space-y-3">
					<h4 class="font-semibold text-slate-900">客戶列表</h4>
					{#each mockClients as client}
						{@const usagePercent = (client.yearlyHours / client.contractHours) * 100}
						<div class="rounded-lg border border-slate-200 p-4 transition-colors hover:border-primary/30">
							<div class="flex items-start justify-between">
								<div>
									<h5 class="font-semibold text-slate-900">{client.name}</h5>
									<p class="text-xs text-slate-500">{client.projectCount} 個專案</p>
								</div>
								<div class="text-right">
									<div class="font-bold text-slate-900">{formatHours(client.monthlyHours)}</div>
									<div class="text-xs text-slate-500">本月</div>
								</div>
							</div>
							<!-- Progress bar -->
							<div class="mt-3">
								<div class="mb-1 flex justify-between text-xs">
									<span class="text-slate-500">年度額度使用</span>
									<span class={usagePercent > 70 ? 'text-amber-600' : 'text-slate-500'}>
										{usagePercent.toFixed(0)}%
									</span>
								</div>
								<div class="h-2 overflow-hidden rounded-full bg-slate-200">
									<div
										class="h-full rounded-full transition-all {usagePercent > 70 ? 'bg-amber-500' : 'bg-primary'}"
										style="width: {Math.min(100, usagePercent)}%"
									></div>
								</div>
								<div class="mt-1 flex justify-between text-xs text-slate-500">
									<span>{formatHours(client.yearlyHours)} 已使用</span>
									<span>剩餘 {formatHours(client.contractHours - client.yearlyHours)}</span>
								</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>
</section>
