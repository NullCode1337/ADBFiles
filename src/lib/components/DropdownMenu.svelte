<script lang="ts">
	import { ChevronDown, Check } from '@lucide/svelte';
	import { slide } from 'svelte/transition';

	let { label, items, activeId, onSelect, icon: Icon, title, onwheel } = $props();
	let showMenu = $state(false);

	function handleSelect(item: unknown) {
		onSelect(item);
		showMenu = false;
	}
</script>

<div class="relative" {onwheel}>
	<button
		onclick={() => (showMenu = !showMenu)}
		class="hover:bg-accent flex items-center gap-1 rounded-md px-2 py-1 text-sm font-semibold transition-colors
               {showMenu ? 'bg-accent' : ''}"
	>
		{#if Icon}<Icon size={14} class="mr-1 opacity-70" />{/if}
		<span class="max-w-[120px] truncate">{label}</span>
		<ChevronDown size={14} class="opacity-50 transition-transform {showMenu ? 'rotate-180' : ''}" />
	</button>

	{#if showMenu}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="fixed inset-0 z-40" onclick={() => (showMenu = false)}></div>

		<div
			class="bg-popover absolute top-full left-0 z-50 mt-1 w-64 rounded-md border p-1 shadow-md"
			transition:slide={{ duration: 100 }}
		>
			<div class="text-muted-foreground px-2 py-1.5 text-[10px] font-bold tracking-wider uppercase">
				{title}
			</div>

			<div class="max-h-[300px] overflow-y-auto">
				{#each items as item (item.id)}
					<button
						class="hover:bg-accent group flex w-full items-center justify-between rounded-sm px-2 py-2 text-left text-sm transition-colors"
						onclick={() => handleSelect(item)}
					>
						<div class="flex flex-col overflow-hidden leading-tight">
							<span class="truncate font-medium">{item.primary}</span>
							<span class="text-muted-foreground truncate text-[10px]">{item.secondary}</span>
						</div>
						{#if activeId === item.id}
							<Check size={14} class="text-primary ml-2 shrink-0" />
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>
