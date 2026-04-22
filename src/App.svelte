<script lang="ts">
  import { Monitor, Smartphone, Terminal, HardDrive, RefreshCw, FolderOpen } from "@lucide/svelte";
  import * as Resizable from "$lib/components/ui/resizable";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";

  let isConnecting = $state(false);
  let desktopPath = $state("~/Documents");
  let androidSerial = $state("Not Connected");

  async function refreshDevices() {
    isConnecting = true;
    setTimeout(() => { isConnecting = false; }, 1000); 
  }
</script>

<div class="h-screen w-screen border overflow-hidden bg-background flex flex-col shadow-2xl">
  <Resizable.PaneGroup direction="horizontal" class="flex-1 overflow-hidden">
    
    <Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
      <div class="flex flex-col h-full bg-muted/5">
        <div class="p-4 border-b bg-background flex items-center justify-between h-14 flex-shrink-0">
          <div class="flex items-center gap-2">
            <Monitor size={18} class="text-blue-500" />
            <span class="font-semibold text-sm">Local Desktop</span>
          </div>
          <span class="text-[10px] text-muted-foreground font-mono truncate max-w-[50%]">{desktopPath}</span>
        </div>
        
        <ScrollArea class="flex-1 p-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            <div class="p-8 border-2 border-dashed rounded-lg flex flex-col items-center justify-center text-muted-foreground hover:border-primary/50 hover:bg-primary/5 transition-all cursor-pointer aspect-square">
              <HardDrive size={24} class="mb-2 opacity-50" />
              <p class="text-xs">Browse Files</p>
            </div>
          </div>
        </ScrollArea>
      </div>
    </Resizable.Pane>

    <Resizable.Handle withHandle />

    <Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
      <div class="flex flex-col h-full">
        <div class="p-4 border-b bg-background flex items-center justify-between h-14 flex-shrink-0">
          <div class="flex items-center gap-2">
            <Smartphone size={18} class="text-green-500" />
            <span class="font-semibold text-sm">Android Device</span>
          </div>
          <Badge variant="outline" class="text-[10px] font-mono truncate max-w-[50%]">{androidSerial}</Badge>
        </div>

        <ScrollArea class="flex-1 flex items-center justify-center relative bg-zinc-950">
          <div class="flex flex-col items-center justify-center h-full w-full text-zinc-500 space-y-4 p-8">
            <FolderOpen size={48} class="mb-4 opacity-20" />
            <p class="text-sm italic text-center">No devices available...</p>
             <Button variant="outline" size="sm" class="mt-4" onclick={refreshDevices}>
                <RefreshCw size={14} class="mr-2" /> Try Reconnecting
            </Button>
          </div>
        </ScrollArea>
      </div>
    </Resizable.Pane>

  </Resizable.PaneGroup>

  <div class="px-4 py-2 border-t bg-muted/30 flex justify-between items-center text-[11px] text-muted-foreground h-10 flex-shrink-0">
    <div class="flex gap-4">
      <span class="hidden sm:inline">Drive</span>
      <span class="hidden md:inline">Storage</span>
    </div>
    <div class="flex items-center gap-1">
      <div class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
      <span>ADB Server</span>
    </div>
  </div>
</div>