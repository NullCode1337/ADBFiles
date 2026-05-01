<script lang="ts">
	import { ArrowRight, ArrowLeft, Lock, Trash2 } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import VirtualList from '$lib/components/VirtualList.svelte';
	import { getFileIcon } from '$lib/icons';
	import { fm } from '$lib/stores/fileManager.svelte';
	import type { FileItem } from '$lib/types';

	interface Props {
		files: FileItem[];
		type: 'desktop' | 'adb';
		onNavigate: (path: string) => Promise<void>;
	}

	const { files, type, onNavigate }: Props = $props();

	function handleClick(file: FileItem) {
		if (file.is_dir && (file.has_permission ?? true)) {
			onNavigate(file.path);
		} else {
			fm.openFile(file, type);
		}
	}
</script>

<VirtualList items={files} itemHeight={38}>
	{#snippet children(file: FileItem)}
		{@const Icon = getFileIcon(file)}
		<div class="group flex h-full items-center gap-1 px-4">
			<button
				onclick={() => handleClick(file)}
				disabled={file.has_permission === false}
				class="flex min-w-0 flex-1 items-center rounded-md p-1.5 text-sm transition-colors
                    {(file.has_permission ?? true)
					? 'hover:bg-accent cursor-pointer'
					: 'cursor-not-allowed opacity-40'}"
			>
				<div class="flex min-w-0 flex-1 items-center gap-3">
					<Icon
						size={16}
						class="shrink-0 {file.is_dir
							? type === 'desktop'
								? 'text-blue-400'
								: 'text-green-400'
							: 'text-zinc-400'}"
					/>
					<span class="block truncate text-left">{file.name}</span>
				</div>
				{#if file.has_permission === false}
					<Lock size={12} class="text-muted-foreground ml-2 shrink-0" />
				{/if}
			</button>

			{#if file.has_permission !== false && file.name !== '..'}
				<div
					class="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100"
				>
					{#if fm.adb.serial}
						<Button
							variant="ghost"
							size="icon"
							class="h-7 w-7 cursor-pointer
                                {type === 'desktop'
								? 'text-blue-500 hover:bg-blue-500/10'
								: 'text-green-500 hover:bg-green-500/10'}"
							onclick={(e) => {
								e.stopPropagation();
								fm.transferFile(file, type);
							}}
							title={type === 'desktop' ? 'Push to Device' : 'Pull from Device'}
						>
							{#if type === 'desktop'}
								<ArrowRight size={14} />
							{:else}
								<ArrowLeft size={14} />
							{/if}
						</Button>
					{/if}

					<Button
						variant="ghost"
						size="icon"
						class="text-destructive h-7 w-7 cursor-pointer hover:text-white"
						onclick={(e) => {
							e.stopPropagation();
							fm.deleteFile(file, type);
						}}
					>
						<Trash2 size={14} />
					</Button>
				</div>
			{/if}
		</div>
	{/snippet}
</VirtualList>
