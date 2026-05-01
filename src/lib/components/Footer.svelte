<script lang="ts">
	import { Eye, EyeOff, VideoIcon, SunIcon, MoonIcon } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { toggleMode } from 'mode-watcher';
	import { fm } from '$lib/stores/fileManager.svelte';
</script>

<div class="bg-muted/30 flex h-10 items-center justify-between border-t px-4 py-2 text-[11px]">
	<!-- Left -->
	<div class="flex items-center gap-4">
		<span>Files: {fm.visibleDesktopFiles.length}</span>
		<Button
			variant="outline"
			size="icon"
			class="h-7 w-7 {fm.showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
			onclick={() => (fm.showHidden = !fm.showHidden)}
		>
			{#if fm.showHidden}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
		</Button>
	</div>

	<!-- Right -->
	<div class="flex items-center gap-2">
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

		<Button onclick={toggleMode} variant="outline" size="icon" class="relative h-7 w-7">
			<SunIcon
				class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-transform duration-200 dark:scale-0 dark:-rotate-90"
			/>
			<MoonIcon
				class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-transform duration-200 dark:scale-100 dark:rotate-0"
			/>
			<span class="sr-only">Toggle theme</span>
		</Button>

		<div class="h-2 w-2 rounded-full {fm.adb.serial ? 'bg-green-500' : 'bg-red-500'}"></div>
		<span class="text-muted-foreground">ADB: {fm.adb.serial ?? 'Not Connected'}</span>
	</div>
</div>
