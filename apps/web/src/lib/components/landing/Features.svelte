<script lang="ts">
	const features = [
		{
			icon: 'zap',
			title: '免打擾追蹤',
			description: '透過 Claude Code 的 statusline hook（狀態列機制）運作，不用手動操作。開始工作就會自動記錄。'
		},
		{
			icon: 'brain',
			title: '智慧閒置偵測',
			description: '監控對話紀錄檔的修改時間，精準判斷是否活躍。休息時自動暫停計時。'
		},
		{
			icon: 'receipt',
			title: '產生計費報表',
			description: '自動計算計費工時，支援手動調整。月報一鍵產生。'
		},
		{
			icon: 'cloud',
			title: '雲端同步儀表板',
			description: 'Turso 雲端資料庫即時同步。用 Web 介面管理客戶、專案、合約。'
		}
	];
</script>

<section class="bg-white px-4 py-20">
	<div class="mx-auto max-w-5xl">
		<!-- Section header -->
		<div class="mb-12 text-center">
			<h2 class="font-heading text-3xl font-bold text-text sm:text-4xl">
				專為 Claude Code 設計
			</h2>
			<p class="mt-4 text-lg text-text-muted">
				從追蹤到報表，一站式解決
			</p>
		</div>

		<!-- Feature cards -->
		<div class="grid gap-6 sm:grid-cols-2">
			{#each features as feature}
				<div class="group cursor-pointer rounded-2xl border border-border bg-slate-50/50 p-6 transition-all hover:border-primary/30 hover:bg-white hover:shadow-lg">
					<!-- Icon -->
					<div class="mb-4 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10 text-primary transition-colors group-hover:bg-primary group-hover:text-white">
						{#if feature.icon === 'zap'}
							<svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
							</svg>
						{:else if feature.icon === 'brain'}
							<svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M12 2a8 8 0 018 8v12l-4-4H8a8 8 0 01-8-8 8 8 0 018-8h4z" />
								<circle cx="8" cy="10" r="1" fill="currentColor" />
								<circle cx="12" cy="10" r="1" fill="currentColor" />
								<circle cx="16" cy="10" r="1" fill="currentColor" />
							</svg>
						{:else if feature.icon === 'receipt'}
							<svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M4 2v20l2-1 2 1 2-1 2 1 2-1 2 1 2-1 2 1V2l-2 1-2-1-2 1-2-1-2 1-2-1-2 1-2-1z" />
								<path d="M8 6h8M8 10h8M8 14h4" />
							</svg>
						{:else if feature.icon === 'cloud'}
							<svg class="h-6 w-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z" />
							</svg>
						{/if}
					</div>

					<!-- Content -->
					<h3 class="mb-2 font-heading text-xl font-semibold text-text">
						{feature.title}
					</h3>
					<p class="text-text-muted">
						{feature.description}
					</p>
				</div>
			{/each}
		</div>
	</div>
</section>
