<script lang="ts">
	import { Monitor } from '@lucide/svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import PartitionMenu from '$lib/components/PartitionMenu.svelte';
	import FileList from '$lib/components/FileList.svelte';
	import { fm } from '$lib/stores/fileManager.svelte';
</script>

<div class="bg-muted/5 flex h-full min-h-0 flex-col">
	<!-- Header -->
	<div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
		<div class="flex items-center gap-2">
			<Monitor size={18} class="text-blue-500" />
			<PartitionMenu />
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
