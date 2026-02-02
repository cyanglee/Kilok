<script lang="ts">
	interface Props {
		githubUrl?: string;
		demoUrl?: string;
	}

	let {
		githubUrl = 'https://github.com/anthropics/claude-code',
		demoUrl = '/demo'
	}: Props = $props();
</script>

<section class="relative overflow-hidden bg-gradient-to-b from-slate-50 via-white to-slate-50">
	<!-- Subtle background pattern -->
	<div class="pointer-events-none absolute inset-0 opacity-40"
		style="background-image: radial-gradient(circle at 1px 1px, rgb(148 163 184 / 0.3) 1px, transparent 0); background-size: 40px 40px;">
	</div>

	<!-- Gradient orbs -->
	<div class="pointer-events-none absolute inset-0">
		<div class="absolute -right-20 top-20 h-[500px] w-[500px] rounded-full bg-blue-200/40 blur-[100px]"></div>
		<div class="absolute -left-20 bottom-40 h-[400px] w-[400px] rounded-full bg-indigo-200/30 blur-[80px]"></div>
	</div>

	<div class="relative mx-auto max-w-7xl px-6 pb-16 pt-28 lg:px-8 lg:pb-24 lg:pt-32">
		<!-- Top badge -->
		<div class="mb-6 flex justify-center lg:mb-8 lg:justify-start">
			<div class="inline-flex items-center gap-2 rounded-full border border-blue-200 bg-blue-50 px-4 py-2 text-sm font-medium text-blue-700">
				<span class="flex h-2 w-2">
					<span class="absolute inline-flex h-2 w-2 animate-ping rounded-full bg-blue-400 opacity-75"></span>
					<span class="relative inline-flex h-2 w-2 rounded-full bg-blue-500"></span>
				</span>
				專為 Claude Code 設計
			</div>
		</div>

		<div class="grid gap-12 lg:grid-cols-2 lg:gap-12">
			<!-- Left: Content -->
			<div class="flex flex-col justify-center">
				<!-- Main headline -->
				<h1 class="text-center font-heading text-3xl font-bold leading-[1.1] tracking-tight text-slate-900 sm:text-4xl lg:text-left lg:text-5xl xl:text-6xl">
					追蹤 AI 協作時間
					<span class="mt-1 block bg-gradient-to-r from-blue-600 via-indigo-600 to-violet-600 bg-clip-text text-transparent sm:mt-2">
						產出專業帳單
					</span>
				</h1>

				<!-- Value props - left aligned on all screens -->
				<div class="mx-auto mt-6 max-w-md space-y-3 text-base text-slate-600 sm:mt-8 sm:space-y-4 sm:text-lg lg:mx-0 lg:max-w-none">
					<p class="flex items-start gap-3">
						<span class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-blue-100 text-blue-600 sm:h-6 sm:w-6">
							<svg class="h-3 w-3 sm:h-3.5 sm:w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
							</svg>
						</span>
						<span>免打擾 — 透過 statusline 自動記錄</span>
					</p>
					<p class="flex items-start gap-3">
						<span class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-indigo-100 text-indigo-600 sm:h-6 sm:w-6">
							<svg class="h-3 w-3 sm:h-3.5 sm:w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
							</svg>
						</span>
						<span>精準計費 — 自動計算工時</span>
					</p>
					<p class="flex items-start gap-3">
						<span class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-violet-100 text-violet-600 sm:h-6 sm:w-6">
							<svg class="h-3 w-3 sm:h-3.5 sm:w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
								<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
							</svg>
						</span>
						<span>客戶報告 — 一鍵分享唯讀連結</span>
					</p>
				</div>

				<!-- CTA buttons -->
				<div class="mt-8 flex flex-col items-center gap-3 sm:flex-row sm:justify-center lg:justify-start">
					<a
						href={githubUrl}
						target="_blank"
						rel="noopener noreferrer"
						class="inline-flex w-full items-center justify-center gap-2 rounded-xl bg-slate-900 px-6 py-3.5 text-base font-semibold text-white shadow-lg shadow-slate-900/20 transition-all hover:bg-slate-800 hover:shadow-xl sm:w-auto"
					>
						<svg class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
							<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
						</svg>
						前往 GitHub
					</a>
					<a
						href={demoUrl}
						class="inline-flex w-full items-center justify-center gap-2 rounded-xl border border-slate-200 bg-white px-6 py-3.5 text-base font-semibold text-slate-700 shadow-sm transition-all hover:border-slate-300 hover:bg-slate-50 sm:w-auto"
					>
						<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
							<path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
						</svg>
						查看示範
					</a>
				</div>
			</div>

			<!-- Right: Dashboard Preview (hidden on mobile) -->
			<div class="relative hidden items-center justify-center lg:flex lg:justify-end">
				<!-- Dashboard mockup -->
				<div class="relative w-full max-w-lg">
					<!-- Shadow/glow -->
					<div class="absolute -inset-4 rounded-2xl bg-gradient-to-br from-blue-500/20 via-indigo-500/10 to-violet-500/20 blur-2xl"></div>

					<!-- Browser chrome -->
					<div class="relative overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl shadow-slate-900/10">
						<div class="flex items-center gap-2 border-b border-slate-100 bg-slate-50 px-4 py-3">
							<div class="flex gap-1.5">
								<div class="h-3 w-3 rounded-full bg-red-400"></div>
								<div class="h-3 w-3 rounded-full bg-amber-400"></div>
								<div class="h-3 w-3 rounded-full bg-green-400"></div>
							</div>
							<div class="mx-auto flex items-center gap-2 rounded-md bg-white px-3 py-1 text-xs text-slate-400 shadow-sm">
								<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
								</svg>
								kilok.app/dashboard
							</div>
						</div>

						<!-- Dashboard content -->
						<div class="p-5">
							<!-- Header -->
							<div class="mb-5 flex items-center justify-between">
								<div>
									<h3 class="text-lg font-bold text-slate-900">客戶報告</h3>
									<p class="text-xs text-slate-500">2026 年 2 月</p>
								</div>
								<span class="rounded-full bg-blue-100 px-3 py-1 text-xs font-medium text-blue-700">
									即時同步
								</span>
							</div>

							<!-- Stats row -->
							<div class="mb-5 grid grid-cols-3 gap-3">
								<div class="rounded-xl bg-slate-50 p-3">
									<div class="text-xs font-medium text-slate-500">本月工時</div>
									<div class="mt-1 text-xl font-bold text-blue-600">12.5h</div>
								</div>
								<div class="rounded-xl bg-slate-50 p-3">
									<div class="text-xs font-medium text-slate-500">計費金額</div>
									<div class="mt-1 text-xl font-bold text-slate-900">$2,500</div>
								</div>
								<div class="rounded-xl bg-slate-50 p-3">
									<div class="text-xs font-medium text-slate-500">追蹤專案</div>
									<div class="mt-1 text-xl font-bold text-indigo-600">3</div>
								</div>
							</div>

							<!-- Client card -->
							<div class="rounded-xl border border-slate-200 bg-white p-4">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-3">
										<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-blue-500 to-violet-500 font-bold text-white">
											A
										</div>
										<div>
											<div class="font-semibold text-slate-900">Acme Corp</div>
											<div class="text-xs text-slate-500">2 個專案 · 合約 96h/年</div>
										</div>
									</div>
									<div class="text-right">
										<div class="font-bold text-slate-900">8.5h</div>
										<div class="text-xs text-slate-500">本月</div>
									</div>
								</div>
								<!-- Progress bar -->
								<div class="mt-3">
									<div class="mb-1 flex justify-between text-xs">
										<span class="text-slate-500">年度額度</span>
										<span class="font-medium text-blue-600">65%</span>
									</div>
									<div class="h-2 overflow-hidden rounded-full bg-slate-100">
										<div class="h-full w-[65%] rounded-full bg-gradient-to-r from-blue-500 to-indigo-500"></div>
									</div>
								</div>
							</div>
						</div>
					</div>

					<!-- Floating statusline badge -->
					<div class="absolute -bottom-8 -left-3 rounded-xl border border-slate-200 bg-slate-900 p-2.5 shadow-lg">
						<div class="flex items-center gap-1.5 text-xs">
							<span class="flex items-center gap-1 rounded-md bg-blue-600 px-2 py-1 font-medium text-white">
								<svg class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<circle cx="12" cy="12" r="10" />
									<polyline points="12 6 12 12 16 14" />
								</svg>
								2h 34m
							</span>
							<span class="rounded-md bg-slate-700 px-2 py-1 text-slate-300">main</span>
							<span class="rounded-md bg-indigo-600 px-2 py-1 font-medium text-white">opus</span>
						</div>
						<div class="mt-1.5 text-[10px] text-slate-400">↑ Claude Code statusline</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</section>
