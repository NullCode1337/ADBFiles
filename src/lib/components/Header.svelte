<script lang="ts">
	import {
		Eye,
		EyeOff,
		VideoIcon,
		SunIcon,
		MoonIcon,
		Smartphone,
		Monitor,
		ChevronRight
	} from '@lucide/svelte';

	import { Separator } from '$lib/components/ui/separator';
	import { Button } from '$lib/components/ui/button';
	import * as Breadcrumb from '$lib/components/ui/breadcrumb';

	import { toggleMode } from 'mode-watcher';
	import { fm } from '$lib/stores/fileManager.svelte';

	let { showDesktop = $bindable() } = $props();
</script>

<header class="flex h-12 shrink-0 items-center justify-between border-b px-4">
	<!-- Left -->
	<div class="flex items-center">
		<Breadcrumb.Root>
			<Breadcrumb.List>
				<Breadcrumb.Item>
					<div class="flex items-center gap-2">
						<Smartphone class="text-muted-foreground size-4" />
						<span class="text-sm font-semibold">ADB</span>
					</div>
				</Breadcrumb.Item>

				<Breadcrumb.Separator>
					<ChevronRight class="size-4" />
				</Breadcrumb.Separator>

				<Breadcrumb.Item>
					<Button
						variant="ghost"
						size="sm"
						class="hover:bg-accent h-7 gap-2 px-2 font-normal"
						onclick={() => (showDesktop = !showDesktop)}
					>
						<Monitor class="size-4" />
						<span class="text-xs">{showDesktop ? 'Hide Desktop' : 'Show Desktop'}</span>
					</Button>
				</Breadcrumb.Item>
			</Breadcrumb.List>
		</Breadcrumb.Root>

		<Separator orientation="vertical" />

		<Button
			variant="ghost"
			size="icon"
			class="h-7 w-7 {fm.showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
			onclick={() => (fm.showHidden = !fm.showHidden)}
			title="Toggle Hidden Files"
		>
			{#if fm.showHidden}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
		</Button>
	</div>

	<!-- Right -->
	<div class="flex items-center gap-4 text-[11px]">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-2">
				<div class="h-2 w-2 rounded-full {fm.adb.serial ? 'bg-green-500' : 'bg-red-500'}"></div>
				<span class="font-medium">{fm.adb.serial ?? 'Disconnected'}</span>
			</div>

			{#if fm.adb.serial}
				<Button
					onclick={fm.startScrcpy}
					variant="outline"
					size="icon"
					class="h-7 w-7 border-green-500/50 text-green-600 hover:bg-green-500/10 dark:text-green-400"
					title="Launch scrcpy"
				>
					<VideoIcon size={14} />
				</Button>
			{/if}

			<Button onclick={toggleMode} variant="ghost" size="icon" class="relative h-7 w-7">
				<SunIcon
					class="h-4 w-4 scale-100 rotate-0 transition-transform duration-200 dark:scale-0 dark:-rotate-90"
				/>
				<MoonIcon
					class="absolute h-4 w-4 scale-0 rotate-90 transition-transform duration-200 dark:scale-100 dark:rotate-0"
				/>
				<span class="sr-only">Toggle theme</span>
			</Button>
		</div>
	</div>
</header>
