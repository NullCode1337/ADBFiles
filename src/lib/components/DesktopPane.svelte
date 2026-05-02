<script lang="ts">
	import { Monitor } from '@lucide/svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import DropdownMenu from '$lib/components/DropdownMenu.svelte';
	import FileList from '$lib/components/FileList.svelte';
	import { fm } from '$lib/stores/fileManager.svelte';

	function scrollPartitions(event: WheelEvent) {
		if (fm.partitions.length <= 1) return;

		const currentPart = fm.partitions
			.filter((p) => fm.desktop.path.startsWith(p.mount_point))
			.sort((a, b) => b.mount_point.length - a.mount_point.length)[0];

		if (!currentPart) return;

		const current = fm.partitions.findIndex((p) => p.mount_point === currentPart.mount_point);
		const direction = event.deltaY > 0 ? 1 : -1;
		const next = (current + direction + fm.partitions.length) % fm.partitions.length;
		const nextPart = fm.partitions[next];

		const path = fm.partitionHistory[nextPart.mount_point] ?? nextPart.mount_point;
		fm.navigateDesktop(path);
	}
</script>

<div class="bg-muted/5 flex h-full min-h-0 flex-col">
	<!-- Header -->
	<div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
		<div class="flex items-center gap-2">
			<DropdownMenu
				label={fm.partLabel}
				title="Local Drives"
				activeId={fm.partitions.find((p) => fm.desktop.path.startsWith(p.mount_point))?.mount_point}
				icon={Monitor}
				onwheel={scrollPartitions}
				items={fm.partitions.map((p) => ({
					id: p.mount_point,
					primary: p.name || 'Local Disk',
					secondary: p.mount_point
				}))}
				onSelect={(item) => fm.navigateDesktop(item.id)}
			/>
		</div>
		<Navigation
			segments={fm.desktop.meta.segments}
			parentPath={fm.desktop.meta.parent}
			onNavigate={fm.navigateDesktop}
			type="desktop"
		/>
	</div>

	<!-- File list -->
	<div class="h-full w-full flex-1 overflow-hidden">
		<FileList files={fm.visibleDesktopFiles} type="desktop" onNavigate={fm.navigateDesktop} />
	</div>
</div>
