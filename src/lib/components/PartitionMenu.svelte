<script lang="ts">
	import { Folder, ChevronDown } from '@lucide/svelte';
	import { fm } from '$lib/stores/fileManager.svelte';

	let showMenu = $state(false);

	async function toggle() {
		if (!showMenu) await fm.fetchPartitions();
		showMenu = !showMenu;
	}

	function select(path: string) {
		showMenu = false;
		fm.navigateDesktop(path);
	}

	function onWheel(event: WheelEvent) {
		if (fm.partitions.length <= 1) return;

		const current =
			fm.partitions
				.map((p, index) => ({ p, index }))
				.filter(({ p }) => fm.desktop.path.startsWith(p.mount_point))
				.sort((a, b) => b.p.mount_point.length - a.p.mount_point.length)[0]?.index ?? 0;

		const direction = event.deltaY > 0 ? 1 : -1;
		const next = (current + direction + fm.partitions.length) % fm.partitions.length;
		const path =
			fm.partitionHistory[fm.partitions[next].mount_point] ?? fm.partitions[next].mount_point;

		fm.navigateDesktop(path);
	}
</script>

<div class="relative">
	<button
		onclick={toggle}
		onwheel={onWheel}
		class="hover:bg-accent flex items-center gap-1 rounded-md px-2 py-1 text-sm font-semibold transition-colors
               {showMenu ? 'bg-accent' : ''}"
	>
		{fm.partLabel}
		<ChevronDown size={14} class="opacity-50 transition-transform {showMenu ? 'rotate-180' : ''}" />
	</button>

	{#if showMenu}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="fixed inset-0 z-40" onclick={() => (showMenu = false)}></div>

		<div class="bg-popover absolute top-full left-0 z-50 mt-1 w-56 rounded-md border p-1 shadow-lg">
			<div class="text-muted-foreground px-2 py-1.5 text-[10px] font-bold uppercase">
				Available Drives
			</div>

			<div class="max-h-[300px] overflow-y-auto">
				{#each fm.partitions as part (part.mount_point)}
					<button
						class="hover:bg-accent flex w-full items-center gap-2 rounded-sm px-2 py-2 text-left text-sm transition-colors"
						onclick={() => select(part.mount_point)}
					>
						<Folder size={14} class="text-blue-400" />
						<div class="flex flex-col leading-tight">
							<span class="truncate">{part.name || 'Local Disk'}</span>
							<span class="text-muted-foreground text-[10px]">{part.mount_point}</span>
						</div>
					</button>
				{/each}

				{#if fm.partitions.length === 0}
					<button
						class="w-full px-2 py-2 text-left text-sm italic opacity-50"
						onclick={() => select('/')}
					>
						Root (/)
					</button>
				{/if}
			</div>
		</div>
	{/if}
</div>
