<script lang="ts">
	import { Smartphone, RefreshCw } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import Navigation from '$lib/components/Navigation.svelte';
	import FileList from '$lib/components/FileList.svelte';
	import DropdownMenu from '$lib/components/DropdownMenu.svelte';
	import { fm } from '$lib/stores/fileManager.svelte';
</script>

<div class="bg-muted/5 flex h-full min-h-0 flex-col">
	<!-- Header -->
	<div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
		<DropdownMenu
			label={fm.adb.name || 'No Device'}
			title="Connected Devices"
			activeId={fm.adb.serial}
			icon={Smartphone}
			items={fm.adb.devices.map((d) => ({
				id: d.serial,
				primary: d.name,
				secondary: `${d.serial} (${d.state})`,
				raw: d
			}))}
			onSelect={(item) => {
				fm.adb.serial = item.id;
				fm.adb.name = item.primary;
				fm.navigateAdb(fm.adb.path);
			}}
			onwheel
		/>

		<div class="flex items-center gap-2">
			<Button variant="ghost" size="icon" class="h-7 w-7" onclick={fm.refreshDevices}>
				<RefreshCw size={14} />
			</Button>
			<Navigation
				segments={fm.adb.meta.segments}
				parentPath={fm.adb.meta.parent}
				onNavigate={fm.navigateAdb}
				type="adb"
				disabled={!fm.adb.serial}
			/>
		</div>
	</div>

	<!-- File list -->
	{#if fm.adb.serial}
		<div class="h-full w-full flex-1 overflow-hidden">
			<FileList files={fm.visibleAdbFiles} type="adb" onNavigate={fm.navigateAdb} />
		</div>
	{:else}
		<div class="flex flex-1 flex-col items-center justify-center gap-2 opacity-40">
			<Smartphone size={40} />
			<p class="text-sm">Connect a device and enable ADB</p>
		</div>
	{/if}
</div>
