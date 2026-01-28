<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<title>Kilok - 工時紀錄</title>
	<meta name="description" content="Kilok 工時追蹤報告系統" />
</svelte:head>

<div class="min-h-screen bg-background">
	<!-- Header -->
	<header class="sticky top-0 z-50 border-b border-border bg-surface/80 backdrop-blur-sm">
		<div class="mx-auto flex h-16 max-w-6xl items-center justify-between px-4">
			<a href="/" class="flex items-center gap-2 text-xl font-bold text-primary">
				<svg class="h-8 w-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<circle cx="12" cy="12" r="10" />
					<polyline points="12 6 12 12 16 14" />
				</svg>
				Kilok
			</a>
			<nav class="flex items-center gap-4">
				<span class="text-sm text-text-muted">紀錄 · Kì-lo̍k</span>
			</nav>
		</div>
	</header>

	<!-- Main content -->
	<main class="mx-auto max-w-6xl px-4 py-8">
		{@render children()}
	</main>

	<!-- Footer -->
	<footer class="border-t border-border py-6 text-center text-sm text-text-muted">
		<p>Kilok &copy; {new Date().getFullYear()} - 工時追蹤報告系統</p>
	</footer>
</div>
