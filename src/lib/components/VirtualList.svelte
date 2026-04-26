<script lang="ts">
	/* eslint-disable @typescript-eslint/no-explicit-any */
	let {
		items,
		itemHeight = 36,
		children
	}: { items: any[]; itemHeight?: number; children: any } = $props();

	let scrollTop = $state(0);
	let containerHeight = $state(0);

	const startIndex = $derived(Math.floor(scrollTop / itemHeight));
	const endIndex = $derived(
		Math.min(items.length - 1, Math.floor((scrollTop + containerHeight) / itemHeight))
	);

	const buffer = 10;
	const visibleStart = $derived(Math.max(0, startIndex - buffer));
	const visibleEnd = $derived(Math.min(items.length - 1, endIndex + buffer));

	const visibleItems = $derived(
		items.slice(visibleStart, visibleEnd + 1).map((item, i) => ({
			data: item,
			index: visibleStart + i
		}))
	);

	const offsetY = $derived(visibleStart * itemHeight);
	const totalHeight = $derived(items.length * itemHeight);

	function handleScroll(e: Event) {
		scrollTop = (e.currentTarget as HTMLElement).scrollTop;
	}
</script>

<div
	class="relative h-full w-full overflow-x-hidden overflow-y-auto"
	bind:clientHeight={containerHeight}
	onscroll={handleScroll}
>
	<div style="height: {totalHeight}px; width: 100%; pointer-events: none;"></div>

	<div class="absolute top-0 left-0 w-full" style="transform: translateY({offsetY}px);">
		{#each visibleItems as { data, index } (data.path || index)}
			<div style="height: {itemHeight}px;">
				{@render children(data)}
			</div>
		{/each}
	</div>
</div>
