<script lang="ts">
  import { 
    Monitor, 
    Smartphone, 
    Folder, 
    File, 
    FileText, 
    ImageIcon, 
    FileCode, 
    Lock, 
    ChevronUp,
    VideoIcon
  } from "@lucide/svelte";
  import * as Resizable from "$lib/components/ui/resizable";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
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

  function getFileIcon(file: FileEntry) {
    if (file.is_dir) return Folder;
    
    const ext = file.name.split('.').pop()?.toLowerCase() ?? "file";
    if (['png', 'jpg', 'gif', 'svg'].includes(ext)) return ImageIcon;
    if (['ts', 'js', 'py', 'rs', 'c', 'cpp', 'json'].includes(ext)) return FileCode;
    if (['txt', 'md', 'pdf', 'doc', 'docx', 'ppt', 'xlsx'].includes(ext)) return FileText;
    if (['mp4', 'wav', 'av1', 'mpeg'].includes(ext)) {
      return VideoIcon;
    } else {
      return File;
    };

  }

  onMount(() => loadDirectory(desktopPath));
</script>

<div class="h-screen w-screen overflow-hidden bg-background flex flex-col">
  <Resizable.PaneGroup direction="horizontal" class="flex-1">
    
    <Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
      <div class="flex flex-col h-full min-h-0 bg-muted/5">
        <div class="p-4 border-b bg-background flex items-center justify-between h-14 shrink-0">
          <div class="flex items-center gap-2">
            <Monitor size={18} class="text-blue-500" />
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
        
        <ScrollArea class="flex-1 h-full w-full">
          <div class="p-4 grid grid-cols-1 gap-1">
            {#each files as file (file.path)}
              {@const Icon = getFileIcon(file)}
              <button 
                onclick={() => file.is_dir && file.has_permission && loadDirectory(file.path)}
                disabled={!file.has_permission}
                class="flex items-center justify-between p-2 rounded-md text-sm transition-colors
                       {file.has_permission ? 'hover:bg-accent cursor-pointer' : 'opacity-40 cursor-not-allowed bg-zinc-100/50'}"
              >
                <div class="flex items-center gap-3">
                  <Icon 
                    size={16} 
                    class={file.is_dir ? "text-blue-400" : "text-zinc-400"} 
                  />
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
      <div class="flex items-center justify-center h-full text-muted-foreground text-sm">
        <div class="flex flex-col items-center gap-2">
          <Smartphone size={40} class="opacity-20" />
          <p>Connect a device via ADB</p>
        </div>
      </div>
    </Resizable.Pane>

  </Resizable.PaneGroup>

  <div class="px-4 py-2 border-t bg-muted/30 flex justify-between items-center text-[11px] h-10">
    <div class="flex gap-4">
      <span>Files: {files.length}</span>
    </div>
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-green-500"></div>
      <span class="text-muted-foreground">ADB Active: {androidSerial}</span>
    </div>
  </div>
</div>