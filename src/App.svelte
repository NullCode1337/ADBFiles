<script lang="ts">
  import { Monitor, Smartphone, RefreshCw, Folder, Lock, ChevronUp } from "@lucide/svelte";
  import * as Resizable from "$lib/components/ui/resizable";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    has_permission: boolean;
  }

  let desktopPath = $state("/");
  let files = $state<FileEntry[]>([]);
  let androidSerial = $state("Not Connected");

  async function loadDirectory(path: string) {
    try {
      const result: FileEntry[] = await invoke("list_directory", { path });
      files = result;
      desktopPath = path;
    } catch (err) {
      console.error("Failed to list directory:", err);
    }
  }

  function goUp() {
    const parts = desktopPath.split('/').filter(Boolean);
    parts.pop();
    loadDirectory("/" + parts.join('/'));
  }

  onMount(() => loadDirectory(desktopPath));
</script>

<div class="h-screen w-screen overflow-hidden bg-background flex flex-col">
  <Resizable.PaneGroup direction="horizontal" class="flex-1">
    
    <Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col border-r">
      <div class="flex flex-col h-full bg-muted/5">
        <div class="p-4 border-b bg-background flex items-center justify-between h-14">
          <div class="flex items-center gap-2">
            <Monitor size={16} class="text-blue-500" />
            <span class="font-semibold text-sm">Local Desktop</span>
          </div>
          <div class="flex items-center gap-2">
             <Button variant="ghost" size="icon" class="h-7 w-7" onclick={goUp}>
                <ChevronUp size={14} />
             </Button>
             <span class="text-[10px] font-mono bg-muted px-2 py-1 rounded truncate max-w-[150px]">
               {desktopPath}
             </span>
          </div>
        </div>
        
        <ScrollArea class="flex-1">
          <div class="p-2 grid grid-cols-1 gap-1">
            {#each files as file}
              <button 
                onclick={() => file.is_dir && file.has_permission && loadDirectory(file.path)}
                disabled={!file.has_permission}
                class="flex items-center justify-between p-2 rounded-md text-sm transition-colors
                       {file.has_permission ? 'hover:bg-accent cursor-pointer' : 'opacity-40 cursor-not-allowed bg-zinc-100/50'}"
              >
                <div class="flex items-center gap-3">
                  <Folder size={16} class={file.is_dir ? "text-blue-400" : "text-zinc-400"} />
                  <span class="truncate">{file.name}</span>
                </div>
                {#if !file.has_permission}
                  <Lock size={12} class="text-muted-foreground" />
                {/if}
              </button>
            {/each}
          </div>
        </ScrollArea>
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
      </Resizable.Pane>

  </Resizable.PaneGroup>

  <div class="px-4 py-2 border-t bg-muted/30 flex justify-between items-center text-[11px] h-10">
    <div class="flex gap-4">
      <span>Files: {files.length}</span>
    </div>
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-green-500"></div>
      <span class="text-muted-foreground">ADB Active</span>
    </div>
  </div>
</div>