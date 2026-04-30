<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Folder, ArrowRight, X } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';

	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	let { adbSerial, onOpen, onPush } = $props<{
		adbSerial: string | null;
		onOpen: (path: string, isDir: boolean) => Promise<void>;
		onPush: (path: string, name: string, isDir: boolean) => Promise<void>;
	}>();

	let pendingDrop = $state<{
		path: string;
		name: string;
		isDir: boolean;
	} | null>(null);

	onMount(() => {
		const unlisten = getCurrentWindow().onDragDropEvent(async (event) => {
			if (event.payload.type === 'drop') {
				const path = event.payload.paths[0];

				const name = path.split(/[\\/]/).filter(Boolean).pop() || path;
				const isDir = await invoke<boolean>('is_directory', { path });

				if (isDir && !adbSerial) {
					await onOpen(path, true);
					return;
				}

				pendingDrop = { path, name, isDir };
			}
		});

		return () => {
			unlisten.then((f) => f());
		};
	});

	function close() {
		pendingDrop = null;
	}
</script>

{#if pendingDrop}
	<div
		class="bg-background/80 fixed inset-0 z-100 flex items-center justify-center backdrop-blur-sm transition-all"
	>
		<div class="bg-card w-full max-w-sm overflow-hidden rounded-xl border shadow-2xl">
			<div class="flex items-center justify-between border-b p-4">
				<h3 class="text-sm font-semibold">External File</h3>
				<Button variant="ghost" size="icon" class="h-6 w-6" onclick={close}>
					<X size={14} />
				</Button>
			</div>

			<div class="p-6 text-center">
				<div class="bg-muted mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full">
					{#if pendingDrop.isDir}
						<Folder class="text-blue-500" size={24} />
					{:else}
						<ArrowRight class="text-green-500" size={24} />
					{/if}
				</div>
				<p class="truncate px-4 text-sm font-medium">{pendingDrop.name}</p>
				<p class="text-muted-foreground mt-1 text-xs">What would you like to do with this item?</p>
			</div>

			<div class="bg-muted/30 flex flex-col gap-2 p-4">
				{#if pendingDrop.isDir}
					<Button
						variant="outline"
						class="w-full cursor-pointer justify-start"
						onclick={() => {
							onOpen(pendingDrop!.path, true);
							close();
						}}
					>
						<Folder size={16} class="mr-2" />
						Open Directory
					</Button>
				{/if}

				<div class={!adbSerial ? 'w-full cursor-not-allowed' : 'w-full'}>
					<Button
						class="w-full cursor-pointer justify-start bg-blue-600 text-white hover:bg-blue-700"
						disabled={!adbSerial}
						onclick={() => {
							onPush(pendingDrop!.path, pendingDrop!.name, pendingDrop!.isDir);
							close();
						}}
					>
						<ArrowRight size={16} class="mr-2" />
						Push to Android
					</Button>
				</div>

				<Button variant="ghost" class="w-full cursor-pointer" onclick={close}>Cancel</Button>
			</div>
		</div>
	</div>
{/if}
