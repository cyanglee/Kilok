<script lang="ts">
	import Navbar from '$lib/components/landing/Navbar.svelte';
	import Hero from '$lib/components/landing/Hero.svelte';
	import Features from '$lib/components/landing/Features.svelte';
	import MockDashboard from '$lib/components/landing/MockDashboard.svelte';
	import NameStory from '$lib/components/landing/NameStory.svelte';
	import Footer from '$lib/components/landing/Footer.svelte';

	// Configuration
	const githubUrl = 'https://github.com/cyanglee/Kilok';
</script>

<svelte:head>
	<title>Kilok - 為 Claude Code 打造的時間追蹤工具</title>
	<meta name="description" content="零侵入、自動記錄、精準計費。讓每一次與 AI 的協作，都有時間的足跡。" />
</svelte:head>

<div class="min-h-screen bg-background">
	<Navbar {githubUrl} />
	<Hero {githubUrl} />
	<Features />
	<MockDashboard />
	<NameStory />
	<Footer {githubUrl} />
</div>
