<script lang="ts">
    import { Smartphone, RefreshCw } from '@lucide/svelte';
    import { Button } from '$lib/components/ui/button';
    import Navigation from '$lib/components/Navigation.svelte';
    import FileList from '$lib/components/FileList.svelte';
    import { fm } from '$lib/stores/fileManager.svelte';
</script>

<div class="bg-muted/5 flex h-full min-h-0 flex-col">
    <!-- Header -->
    <div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
        <div class="flex items-center gap-2">
            <Smartphone size={18} class="text-green-500" />
            <span class="text-sm font-semibold">Android Device</span>
        </div>
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
            <FileList
                files={fm.visibleAdbFiles}
                type="adb"
                onNavigate={fm.navigateAdb}
            />
        </div>
    {:else}
        <div class="flex flex-1 flex-col items-center justify-center gap-2 opacity-40">
            <Smartphone size={40} />
            <p class="text-sm">Connect a device and enable ADB</p>
        </div>
    {/if}
</div>
