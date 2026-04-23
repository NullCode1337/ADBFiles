<script lang="ts">
  import { Monitor, Smartphone, RefreshCw, Folder, File, FileText, ImageIcon, FileCode, Lock, SunIcon, MoonIcon, ChevronUp, VideoIcon, Eye, EyeOff } from "@lucide/svelte";
  
  import * as Resizable from "$lib/components/ui/resizable";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  
  import { invoke } from "@tauri-apps/api/core";
  import { ModeWatcher, toggleMode } from "mode-watcher";
  import { onMount } from "svelte";

  interface FileEntry {
    name: string;
    path: string;
    is_dir: boolean;
    has_permission: boolean;
  }
  
  interface AdbFile {
    name: string;
    is_dir: boolean;
  }

  let desktopPath = $state(localStorage.getItem("lastDesktopPath") ?? "/");

  let showHidden = $state(false);
  let files = $state<FileEntry[]>([]);
  let visibleFiles = $derived(
    showHidden ? files : files.filter(f => !f.name.startsWith('.'))
  );

  let selectedSerial = $state<string | null>(null);
  let adbDevices = $state<any[]>([]);
  let adbFiles = $state<AdbFile[]>([]);
  let adbPath = $state("/storage/emulated/0/");

  $effect(() => {
    localStorage.setItem("lastDesktopPath", desktopPath);
  });

  async function loadDirectory(path: string) {
    try {
      const result: FileEntry[] = await invoke("list_directory", { path });
      files = result;
      desktopPath = path;
    } catch (err) {
      console.error("Failed to list directory:", err);
      if (path !== "/") {
        console.warn("Path not found, reverting to root");
        loadDirectory("/");
      }
    }
  }

  async function refreshDevices() {
    try {
      adbDevices = await invoke("list_adb_devices");
      if (adbDevices.length > 0 && !selectedSerial) {
        selectedSerial = adbDevices[0].serial;
        loadAdbDirectory(adbPath);
      } else if (adbDevices.length === 0) {
        selectedSerial = null;
        adbFiles = [];
      }
    } catch (e) {
      console.error("ADB Error:", e);
    }
  }

  async function loadAdbDirectory(path: string) {
    if (!selectedSerial) return;
    try {
      let cleanPath = path.startsWith('/') ? path : '/' + path;
      if (cleanPath.length > 1 && cleanPath.endsWith('/')) {
        cleanPath = cleanPath.slice(0, -1);
      }
      
      const result: AdbFile[] = await invoke("list_adb_directory", { 
        serial: selectedSerial, 
        path: cleanPath 
      });
      
      adbFiles = result.sort((a, b) => Number(b.is_dir) - Number(a.is_dir) || a.name.localeCompare(b.name));
      adbPath = cleanPath;
    } catch (err) {
      console.error("ADB Load Error:", err);
    }
  }

  function joinAdbPath(base: string, name: string) {
    const cleanBase = base.endsWith('/') ? base : base + '/';
    return cleanBase + name;
  }

  function getFileIcon(file: FileEntry) {
    if (file.is_dir) return Folder;
    
    const ext = file.name.split('.').pop()?.toLowerCase() ?? "file";
    if (['png', 'jpg', 'gif', 'svg'].includes(ext)) return ImageIcon;
    if (['ts', 'js', 'py', 'rs', 'c', 'cpp', 'json'].includes(ext)) return FileCode;
    if (['txt', 'md', 'pdf', 'doc', 'docx', 'ppt', 'xlsx'].includes(ext)) return FileText;
    if (['mp4', 'wav', 'av1', 'mpeg'].includes(ext)) return VideoIcon;
    return File;
  }

  onMount(() => {
    refreshDevices();
    loadDirectory(desktopPath);
  });
</script>

<ModeWatcher />

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
              <Button 
                variant="ghost" 
                size="icon" 
                class="h-7 w-7 {showHidden ? 'text-blue-500' : 'text-muted-foreground'}" 
                onclick={() => showHidden = !showHidden}
                title={showHidden ? "Hide Hidden Files" : "Show Hidden Files"}
              >
                {#if showHidden}
                  <Eye size={14} />
                {:else}
                  <EyeOff size={14} />
                {/if}
              </Button>
             <Button variant="ghost" size="icon" class="h-7 w-7" onclick={() => {
                const parts = desktopPath.split('/').filter(Boolean);
                parts.pop();
                loadDirectory("/" + parts.join('/'));
             }}>
                <ChevronUp size={14} />
             </Button>
             <span class="text-[10px] font-mono bg-muted px-2 py-1 rounded truncate max-w-[150px]">
               {desktopPath}
             </span>
          </div>
        </div>
        
        <ScrollArea class="flex-1 h-full w-full">
          <div class="p-4 grid grid-cols-1 gap-1">
            {#each visibleFiles as file (file.path)}
              {@const Icon = getFileIcon(file)}
              <button 
                onclick={() => file.is_dir && file.has_permission && loadDirectory(file.path)}
                disabled={!file.has_permission}
                class="flex items-center justify-between p-2 rounded-md text-sm transition-colors
                       {file.has_permission ? 'hover:bg-accent cursor-pointer' : 'opacity-40 cursor-not-allowed'}"
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
      <div class="flex flex-col h-full min-h-0 bg-muted/5">
        <div class="p-4 border-b bg-background flex items-center justify-between h-14 shrink-0">
          <div class="flex items-center gap-2">
            <Smartphone size={18} class="text-green-500" />
            <span class="font-semibold text-sm">Android Device</span>
          </div>
          
          <div class="flex items-center gap-2">
            <Button variant="ghost" size="icon" class="h-7 w-7" onclick={refreshDevices}>
              <RefreshCw size={14} />
            </Button>
            <Button 
              variant="ghost" 
              size="icon" 
              class="h-7 w-7" 
              onclick={() => {
                if (adbPath === "/" || adbPath === "") return;
                const parts = adbPath.split('/').filter(Boolean);
                parts.pop();
                const newPath = "/" + parts.join('/');
                loadAdbDirectory(newPath);
              }} 
              disabled={!selectedSerial || adbPath === "/"}
            >
              <ChevronUp size={14} />
            </Button>
            <span class="text-[10px] font-mono bg-muted px-2 py-1 rounded truncate max-w-[150px]">
              {adbPath}
            </span>
          </div>
        </div>

        {#if selectedSerial}
          <ScrollArea class="flex-1 h-full w-full">
            <div class="p-4 grid grid-cols-1 gap-1">
              {#each adbFiles as file (file.name)}
                <button 
                  onclick={() => file.is_dir && loadAdbDirectory(joinAdbPath(adbPath, file.name))}
                  class="flex items-center gap-3 p-2 rounded-md text-sm hover:bg-accent transition-colors"
                >
                  {#if file.is_dir}
                    <Folder size={16} class="text-green-400" />
                  {:else}
                    <File size={16} class="text-zinc-400" />
                  {/if}
                  <span class="truncate">{file.name}</span>
                </button>
              {/each}
            </div>
          </ScrollArea>
        {:else}
          <div class="flex-1 flex flex-col items-center justify-center gap-2 opacity-40">
            <Smartphone size={40} />
            <p class="text-sm">Connect a device and enable ADB</p>
          </div>
        {/if}
      </div>
    </Resizable.Pane>

  </Resizable.PaneGroup>

  <div class="px-4 py-2 border-t bg-muted/30 flex justify-between items-center text-[11px] h-10">
    <div class="flex gap-4">
      <span>Files: {visibleFiles.length}</span>
    </div>
    <div class="flex items-center gap-2">
      <Button onclick={toggleMode} variant="outline" size="icon">
        <SunIcon
          class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-all! dark:scale-0 dark:-rotate-90"
        />
        <MoonIcon
          class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-all! dark:scale-100 dark:rotate-0"
        />
      </Button>
      <div class="w-2 h-2 rounded-full bg-green-500"></div>
      <span class="text-muted-foreground">ADB: {selectedSerial ?? "Not Connected"}</span>
    </div>
  </div>
</div>