<script lang="ts">
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { Folder, ArrowRight, X } from "@lucide/svelte";
    import { Button } from "$lib/components/ui/button";
    import { onMount } from "svelte";

    let { adbPath, adbSerial, onOpen, onPush } = $props<{
        adbPath: string;
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
        const unlisten = getCurrentWindow().onDragDropEvent((event) => {
            if (event.payload.type === 'drop') {
                const path = event.payload.paths[0];
                const name = path.split(/[/\\]/).pop() || path;
                const isDir = !name.includes('.'); 

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
    <div class="fixed inset-0 z-100 flex items-center justify-center bg-background/80 backdrop-blur-sm transition-all">
        <div class="bg-card w-full max-w-sm rounded-xl border shadow-2xl overflow-hidden">
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
                <p class="text-sm font-medium truncate px-4">{pendingDrop.name}</p>
                <p class="text-muted-foreground mt-1 text-xs">What would you like to do with this item?</p>
            </div>

            <div class="bg-muted/30 flex flex-col gap-2 p-4">
                {#if pendingDrop.isDir}
                    <Button 
                        variant="outline" 
                        class="w-full justify-start" 
                        onclick={() => { onOpen(pendingDrop!.path, true); close(); }}
                    >
                        <Folder size={16} class="mr-2" />
                        Open Directory
                    </Button>
                {/if}

                <Button 
                    class="w-full justify-start bg-blue-600 hover:bg-blue-700 text-white" 
                    disabled={!adbSerial}
                    onclick={() => { onPush(pendingDrop!.path, pendingDrop!.name, pendingDrop!.isDir); close(); }}
                >
                    <ArrowRight size={16} class="mr-2" />
                    Push to Android {adbPath !== "/storage/emulated/0" ? `(${adbPath})` : ""}
                </Button>

                <Button variant="ghost" class="w-full" onclick={close}>
                    Cancel
                </Button>
            </div>
        </div>
    </div>
{/if}