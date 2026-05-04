<script lang="ts">
	import * as Resizable from '$lib/components/ui/resizable';

	import { ModeWatcher } from 'mode-watcher';
	import { onMount } from 'svelte';

	import { fm } from '$lib/stores/fileManager.svelte';

	import FileDropper from '$lib/components/FileDropper.svelte';
	import DesktopPane from '$lib/components/DesktopPane.svelte';
	import AdbPane from '$lib/components/AdbPane.svelte';
	import Header from '$lib/components/Header.svelte';

	let showDesktop = $state(
		typeof window !== 'undefined' ? localStorage.getItem('showDesktop') === 'true' : false
	);

	$effect(() => {
		localStorage.setItem('showDesktop', String(showDesktop));
	});

	$effect(() => {
		const serial = fm.activeDevice?.serial ?? null;
		if (serial !== fm.adb.serial) {
			fm.adb.serial = serial;
			if (serial) fm.navigateAdb(fm.adb.path);
			else fm.adb.files = [];
		}
	});

	onMount(() => {
		let cleanup: (() => void) | undefined;

		fm.init().then((unlisten) => {
			cleanup = unlisten;
		});

		return () => cleanup?.();
	});
</script>

<ModeWatcher />

<FileDropper adbSerial={fm.adb.serial} onOpen={fm.dropOpen} onPush={fm.dropPush} />

<div class="bg-background flex h-screen w-screen flex-col overflow-hidden">
	<Header bind:showDesktop />

	<Resizable.PaneGroup direction="horizontal" class="flex-1">
		{#if showDesktop}
			<Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
				<DesktopPane />
			</Resizable.Pane>
			<Resizable.Handle withHandle />
		{/if}

		<Resizable.Pane defaultSize={showDesktop ? 50 : 100} minSize={30} class="flex flex-col">
			<AdbPane />
		</Resizable.Pane>
	</Resizable.PaneGroup>
</div>
