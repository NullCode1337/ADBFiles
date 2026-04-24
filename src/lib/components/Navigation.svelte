<script lang="ts">
	import { Folder, ChevronUp } from '@lucide/svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { Button } from '$lib/components/ui/button';

	let { currentPath, segments, onNavigate, type, disabled = false } = $props();

	const alias_zero = (name: string) => (type === 'adb' && name === '0' ? 'sdcard' : name);

	function goUp() {
		const win = currentPath.includes('\\');
		const sep = win ? '\\' : '/';

		let cleanPath = currentPath;
		if (cleanPath.endsWith(sep) && cleanPath.length > 3) {
			cleanPath = cleanPath.slice(0, -1);
		}

		const parts = cleanPath.split(sep).filter(Boolean);

		if (parts.length <= 1) {
			onNavigate(win ? parts[0] + '\\' : '/');
			return;
		}

		parts.pop();
		let newPath = parts.join(sep);
		if (win && newPath.endsWith(':')) {
			newPath += '\\';
		} else if (!win && newPath === '') {
			newPath = '/';
		}

		onNavigate(newPath);
	}

	const displayLabel = $derived(() => {
		const win = currentPath.includes('\\');
		const sep = win ? '\\' : '/';
		if (currentPath === '/' || (win && currentPath.length <= 3)) return 'root';
		const lastPart = currentPath.split(sep).filter(Boolean).pop();
		return alias_zero(lastPart || 'root');
	});
</script>

<div class="flex items-center gap-2">
	<Button variant="ghost" size="icon" class="h-7 w-7" {disabled} onclick={goUp}>
		<ChevronUp size={14} />
	</Button>

	<DropdownMenu.Root>
		<DropdownMenu.Trigger {disabled}>
			{#snippet child({ props })}
				<Button
					{...props}
					variant="secondary"
					class="h-7 max-w-37.5 truncate px-2 font-mono text-[10px]"
				>
					{displayLabel()}
				</Button>
			{/snippet}
		</DropdownMenu.Trigger>

		<DropdownMenu.Content align="end" class="w-48">
			<DropdownMenu.Group>
				<DropdownMenu.Label>Jump to folder</DropdownMenu.Label>
				<DropdownMenu.Separator />
				{#each segments as segment (segment.path)}
					<DropdownMenu.Item
						onclick={() => onNavigate(segment.path)}
						class="flex cursor-pointer items-center gap-2"
					>
						<Folder size={14} class={type === 'desktop' ? 'text-blue-400' : 'text-green-400'} />
						<span class="truncate text-xs">{alias_zero(segment.name)}</span>
					</DropdownMenu.Item>
				{/each}
			</DropdownMenu.Group>
		</DropdownMenu.Content>
	</DropdownMenu.Root>
</div>
