<script lang="ts">
  	import { Monitor, Smartphone, RefreshCw, Folder, File, FileText, ImageIcon, FileCode, Lock, SunIcon, MoonIcon, ChevronUp, VideoIcon, Eye, EyeOff } from "@lucide/svelte";

	import * as Resizable from '$lib/components/ui/resizable';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Button } from '$lib/components/ui/button';

	import { invoke } from '@tauri-apps/api/core';
	import { ModeWatcher, toggleMode } from 'mode-watcher';
	import { onMount } from 'svelte';

	interface File {
		name: string;
		path: string;
		is_dir: boolean;
		has_permission?: boolean;
	}

	interface DeviceObj {
		serial: string;
		state: string;
	}

	let desktop = $state({
		path: localStorage.getItem('lastDesktopPath') ?? '/',
		files: [] as File[],
		showHidden: false
	});

	let adb = $state({
		path: '/storage/emulated/0',
		files: [] as File[],
		serial: null as string | null,
		devices: [] as DeviceObj[]
	});

	function createSegments(path: string) {
		return path
			.split('/')
			.filter(Boolean)
			.reduce(
				(acc, curr, i, arr) => {
					const fullPath = '/' + arr.slice(0, i + 1).join('/');
					acc.push({ name: curr, path: fullPath });
					return acc;
				},
				[{ name: 'root', path: '/' }]
			);
	}

	const desktopSegments = $derived(createSegments(desktop.path));
	const visibleDesktopFiles = $derived(
		desktop.showHidden ? desktop.files : desktop.files.filter((f) => !f.name.startsWith('.'))
	);

	function getFileIcon(file: File) {
		if (file.is_dir) return Folder;
		const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
		const maps = {
			image: ['png', 'jpg', 'gif', 'svg'],
			code: ['ts', 'js', 'py', 'rs', 'c', 'cpp', 'json'],
			text: ['txt', 'md', 'pdf', 'doc', 'docx', 'ppt', 'xlsx'],
			video: ['mp4', 'wav', 'av1', 'mpeg']
		};
		if (maps.image.includes(ext)) return ImageIcon;
		if (maps.code.includes(ext)) return FileCode;
		if (maps.text.includes(ext)) return FileText;
		if (maps.video.includes(ext)) return VideoIcon;
		return File;
	}

	async function navigateDesktop(path: string) {
		try {
			const result = await invoke<File[]>('list_directory', { path });
			desktop.files = result;
			desktop.path = path;
			localStorage.setItem('lastDesktopPath', path);
		} catch (err) {
			console.error(err);
		}
	}

	async function navigateAdb(path: string) {
		if (!adb.serial) return;
		try {
			const result = await invoke<File[]>('list_adb_directory', { serial: adb.serial, path });
			adb.files = result;
			adb.path = path;
		} catch (err) {
			console.error(err);
		}
	}

	async function refreshDevices() {
		try {
			const devices = await invoke<DeviceObj[]>('list_adb_devices');
			adb.devices = devices;
			if (devices.length > 0) {
				adb.serial = devices[0].serial;
				await navigateAdb(adb.path);
			} else {
				adb.serial = null;
				adb.files = [];
			}
		} catch (e) {
			console.error('Failed to list devices:', e);
		}
	}

	async function startScrcpy() {
		if (!adb.serial) return;

		try {
			await invoke('launch_scrcpy', { serial: adb.serial });
		} catch (err) {
			console.error('Scrcpy Error:', err);
		}
	}

	onMount(() => {
		refreshDevices();
		navigateDesktop(desktop.path);
	});
</script>

<ModeWatcher />

{#snippet file_list(
    files: File[], 
    onNavigate: (path: string) => Promise<void>, 
    type: 'desktop' | 'adb'
)}
	<div class="grid grid-cols-1 gap-1 p-4">
		{#each files as file (file.path)}
			{@const Icon = getFileIcon(file)}
			<button
				onclick={() => file.is_dir && (file.has_permission ?? true) && onNavigate(file.path)}
				disabled={file.has_permission === false}
				class="flex items-center justify-between rounded-md p-2 text-sm transition-colors
                {(file.has_permission ?? true)
					? 'hover:bg-accent cursor-pointer'
					: 'cursor-not-allowed opacity-40'}"
			>
				<div class="flex items-center gap-3">
					<Icon
						size={16}
						class={file.is_dir
							? type === 'desktop'
								? 'text-blue-400'
								: 'text-green-400'
							: 'text-zinc-400'}
					/>
					<span class="truncate">{file.name}</span>
				</div>
				{#if file.has_permission === false}
					<Lock size={12} class="text-muted-foreground" />
				{/if}
			</button>
		{/each}
	</div>
{/snippet}

<div class="bg-background flex h-screen w-screen flex-col overflow-hidden">
	<Resizable.PaneGroup direction="horizontal" class="flex-1">
		<!-- #region Desktop pane -->
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
							class="h-7 w-7 {desktop.showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
							onclick={() => (desktop.showHidden = !desktop.showHidden)}
							title={desktop.showHidden ? 'Hide Hidden Files' : 'Show Hidden Files'}
						>
							{#if desktop.showHidden}
								<Eye size={14} />
							{:else}
								<EyeOff size={14} />
							{/if}
						</Button>
						<Button
							variant="ghost"
							size="icon"
							onclick={() => {
								const parts = desktop.path.split('/').filter(Boolean);
								parts.pop();
								navigateDesktop('/' + parts.join('/'));
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
										{desktop.path === '/' ? 'root' : desktop.path.split('/').pop()}
									</Button>
								{/snippet}
							</DropdownMenu.Trigger>

							<DropdownMenu.Content align="end" class="w-48">
								<DropdownMenu.Group>
									<DropdownMenu.Label>Jump to folder</DropdownMenu.Label>
									<DropdownMenu.Separator />
									{#each desktopSegments as segment (segment.path)}
										<DropdownMenu.Item
											onclick={() => navigateDesktop(segment.path)}
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
					{@render file_list(visibleDesktopFiles, navigateDesktop, 'desktop')}
				</ScrollArea>
			</div>
		</Resizable.Pane>
		<!-- #endregion -->

		<Resizable.Handle withHandle />

		<!-- #region ADB pane -->
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
							onclick={() => {
								const parts = adb.path.split('/').filter(Boolean);
								parts.pop();
								navigateAdb('/' + parts.join('/'));
							}}
							disabled={!adb.serial || adb.path === '/'}
						>
							<ChevronUp size={14} />
						</Button>
						<span class="bg-muted max-w-[150px] truncate rounded px-2 py-1 font-mono text-[10px]">
							{adb.path}
						</span>
					</div>
				</div>

				{#if adb.serial}
					<ScrollArea class="h-full w-full flex-1">
						{@render file_list(adb.files, navigateAdb, 'adb')}
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
	<!-- #endregion -->

	<!-- #region Footer -->
	<div class="bg-muted/30 flex h-10 items-center justify-between border-t px-4 py-2 text-[11px]">
		<div class="flex gap-4">
			<span>Files: {visibleDesktopFiles.length}</span>
		</div>
		<div class="flex items-center gap-2">
			{#if adb.serial}
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
					class="h-[1.2rem] w-[1.2rem] scale-100 rotate-0 transition-transform duration-200 dark:scale-0 dark:-rotate-90"
				/>
				<MoonIcon
					class="absolute h-[1.2rem] w-[1.2rem] scale-0 rotate-90 transition-transform duration-200 dark:scale-100 dark:rotate-0"
				/>
				<span class="sr-only">Toggle theme</span>
			</Button>
			<div class="h-2 w-2 rounded-full bg-green-500"></div>
			<span class="text-muted-foreground">ADB: {adb.serial ?? 'Not Connected'}</span>
		</div>
	</div>
	<!-- #endregion -->
</div>
