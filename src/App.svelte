<script lang="ts">
  	import { Monitor, Smartphone, RefreshCw, Folder, File, FileText, ImageIcon, FileCode, Lock, SunIcon, MoonIcon, ChevronUp, VideoIcon, Eye, EyeOff } from "@lucide/svelte";

	import * as Resizable from '$lib/components/ui/resizable';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Button } from '$lib/components/ui/button';

	import { invoke } from '@tauri-apps/api/core';
	import { ModeWatcher, toggleMode } from 'mode-watcher';
	import { onMount } from 'svelte';

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

	let desktopPath = $state(localStorage.getItem('lastDesktopPath') ?? '/');

	let showHidden = $state(false);
	let files = $state<FileEntry[]>([]);
	let visibleFiles = $derived(showHidden ? files : files.filter((f) => !f.name.startsWith('.')));

	let selectedSerial = $state<string | null>(null);
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let adbDevices = $state<any[]>([]);
	let adbFiles = $state<AdbFile[]>([]);
	let adbPath = $state('/storage/emulated/0/');

	$effect(() => {
		localStorage.setItem('lastDesktopPath', desktopPath);
	});

	let pathSegments = $derived(
		desktopPath
			.split('/')
			.filter(Boolean)
			.reduce(
				(acc, curr, i, arr) => {
					const fullPath = '/' + arr.slice(0, i + 1).join('/');
					acc.push({ name: curr, path: fullPath });
					return acc;
				},
				[{ name: 'root', path: '/' }] as { name: string; path: string }[]
			)
	);

	async function loadDirectory(path: string) {
		try {
			const result: FileEntry[] = await invoke('list_directory', { path });
			files = result;
			desktopPath = path;
		} catch (err) {
			console.error('Failed to list directory:', err);
			if (path !== '/') {
				console.warn('Path not found, reverting to root');
				loadDirectory('/');
			}
		}
	}

	async function refreshDevices() {
		try {
			adbDevices = await invoke('list_adb_devices');
			if (adbDevices.length > 0 && !selectedSerial) {
				selectedSerial = adbDevices[0].serial;
				loadAdbDirectory(adbPath);
			} else if (adbDevices.length === 0) {
				selectedSerial = null;
				adbFiles = [];
			}
		} catch (e) {
			console.error('ADB Error:', e);
		}
	}

	async function loadAdbDirectory(path: string) {
		if (!selectedSerial) return;
		try {
			let cleanPath = path.startsWith('/') ? path : '/' + path;
			if (cleanPath.length > 1 && cleanPath.endsWith('/')) {
				cleanPath = cleanPath.slice(0, -1);
			}

			const result: AdbFile[] = await invoke('list_adb_directory', {
				serial: selectedSerial,
				path: cleanPath
			});

			adbFiles = result.sort(
				(a, b) => Number(b.is_dir) - Number(a.is_dir) || a.name.localeCompare(b.name)
			);
			adbPath = cleanPath;
		} catch (err) {
			console.error('ADB Load Error:', err);
		}
	}

	async function startScrcpy() {
        if (!selectedSerial) return;
        
        try {
            await invoke('launch_scrcpy', { serial: selectedSerial });
        } catch (err) {
            console.error('Scrcpy Error:', err);
        }
    }

	function joinAdbPath(base: string, name: string) {
		const cleanBase = base.endsWith('/') ? base : base + '/';
		return cleanBase + name;
	}

	function getFileIcon(file: FileEntry) {
		if (file.is_dir) return Folder;

		const ext = file.name.split('.').pop()?.toLowerCase() ?? 'file';
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

<div class="bg-background flex h-screen w-screen flex-col overflow-hidden">
	<Resizable.PaneGroup direction="horizontal" class="flex-1">
		<Resizable.Pane defaultSize={50} minSize={30} class="flex flex-col">
			<div class="bg-muted/5 flex h-full min-h-0 flex-col">
				<div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
					<div class="flex items-center gap-2">
						<Monitor size={18} class="text-blue-500" />
						<span class="text-sm font-semibold">Local Desktop</span>
					</div>
					<div class="flex items-center gap-2">
						<Button
							variant="ghost"
							size="icon"
							class="h-7 w-7 {showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
							onclick={() => (showHidden = !showHidden)}
							title={showHidden ? 'Hide Hidden Files' : 'Show Hidden Files'}
						>
							{#if showHidden}
								<Eye size={14} />
							{:else}
								<EyeOff size={14} />
							{/if}
						</Button>
						<Button
							variant="ghost"
							size="icon"
							class="h-7 w-7"
							onclick={() => {
								const parts = desktopPath.split('/').filter(Boolean);
								parts.pop();
								loadDirectory('/' + parts.join('/'));
							}}
						>
							<ChevronUp size={14} />
						</Button>
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>
								{#snippet child({ props })}
									<Button
										{...props}
										variant="secondary"
										class="h-7 max-w-[150px] truncate px-2 font-mono text-[10px]"
									>
										{desktopPath === '/' ? 'root' : desktopPath.split('/').pop()}
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>

							<DropdownMenu.Content align="end" class="w-48">
								<DropdownMenu.Group>
									<DropdownMenu.Label>Jump to folder</DropdownMenu.Label>
									<DropdownMenu.Separator />
									{#each pathSegments as segment (segment.path)}
										<DropdownMenu.Item
											onclick={() => loadDirectory(segment.path)}
											class="flex cursor-pointer items-center gap-2"
										>
											<Folder size={14} class="text-blue-400" />
											<span class="truncate text-xs">{segment.name}</span>
										</DropdownMenu.Item>
									{/each}
								</DropdownMenu.Group>
							</DropdownMenu.Content>
						</DropdownMenu.Root>
					</div>
				</div>

				<ScrollArea class="h-full w-full flex-1">
					<div class="grid grid-cols-1 gap-1 p-4">
						{#each visibleFiles as file (file.path)}
							{@const Icon = getFileIcon(file)}
							<button
								onclick={() => file.is_dir && file.has_permission && loadDirectory(file.path)}
								disabled={!file.has_permission}
								class="flex items-center justify-between rounded-md p-2 text-sm transition-colors
                       {file.has_permission
									? 'hover:bg-accent cursor-pointer'
									: 'cursor-not-allowed opacity-40'}"
							>
								<div class="flex items-center gap-3">
									<Icon size={16} class={file.is_dir ? 'text-blue-400' : 'text-zinc-400'} />
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
			<div class="bg-muted/5 flex h-full min-h-0 flex-col">
				<div class="bg-background flex h-14 shrink-0 items-center justify-between border-b p-4">
					<div class="flex items-center gap-2">
						<Smartphone size={18} class="text-green-500" />
						<span class="text-sm font-semibold">Android Device</span>
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
								if (adbPath === '/' || adbPath === '') return;
								const parts = adbPath.split('/').filter(Boolean);
								parts.pop();
								const newPath = '/' + parts.join('/');
								loadAdbDirectory(newPath);
							}}
							disabled={!selectedSerial || adbPath === '/'}
						>
							<ChevronUp size={14} />
						</Button>
						<span class="bg-muted max-w-[150px] truncate rounded px-2 py-1 font-mono text-[10px]">
							{adbPath}
						</span>
					</div>
				</div>

				{#if selectedSerial}
					<ScrollArea class="h-full w-full flex-1">
						<div class="grid grid-cols-1 gap-1 p-4">
							{#each adbFiles as file (file.name)}
								<button
									onclick={() => file.is_dir && loadAdbDirectory(joinAdbPath(adbPath, file.name))}
									class="hover:bg-accent flex items-center gap-3 rounded-md p-2 text-sm transition-colors"
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
					<div class="flex flex-1 flex-col items-center justify-center gap-2 opacity-40">
						<Smartphone size={40} />
						<p class="text-sm">Connect a device and enable ADB</p>
					</div>
				{/if}
			</div>
		</Resizable.Pane>
	</Resizable.PaneGroup>

	<div class="bg-muted/30 flex h-10 items-center justify-between border-t px-4 py-2 text-[11px]">
		<div class="flex gap-4">
			<span>Files: {visibleFiles.length}</span>
		</div>
		<div class="flex items-center gap-2">
			{#if selectedSerial}
				<Button 
					onclick={startScrcpy} 
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
					class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-transform duration-200 dark:-rotate-90 dark:scale-0"
				/>
				<MoonIcon
					class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-transform duration-200 dark:rotate-0 dark:scale-100"
				/>
				<span class="sr-only">Toggle theme</span>
			</Button>
			<div class="h-2 w-2 rounded-full bg-green-500"></div>
			<span class="text-muted-foreground">ADB: {selectedSerial ?? 'Not Connected'}</span>
		</div>
	</div>
</div>
