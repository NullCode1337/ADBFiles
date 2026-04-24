<script lang="ts">
  	import { Monitor, Smartphone, RefreshCw, Folder, File, FileText, ImageIcon, FileCode, Lock, SunIcon, MoonIcon, VideoIcon, Eye, EyeOff, ChevronDown } from "@lucide/svelte";

	import * as Resizable from '$lib/components/ui/resizable';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Button } from '$lib/components/ui/button';

	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { ModeWatcher, toggleMode } from 'mode-watcher';
	import { onMount } from 'svelte';

	import Navigation from '$lib/components/Navigation.svelte';

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

	interface Partition {
		name: string;
		mount_point: string;
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

	let partitions = $state([] as Partition[]);
	let showPartitionMenu = $state(false);

	function createSegments(path: string, type: 'desktop' | 'adb') {
		const normalized = path.replace(/\\/g, '/');
		const parts = normalized.split('/').filter(Boolean);

		const win = path.includes(':\\') || path.match(/^[a-zA-Z]:/);
		const rootName = win && parts.length > 0 ? parts[0] : 'root';
		const rootPath = win && parts.length > 0 ? parts[0] + '\\' : '/';

		const segments = parts
			.map((curr, i) => {
				if (win && i === 0) return null;
				const subParts = parts.slice(0, i + 1);
				let fullPath;

				if (type === 'desktop' && win) {
					fullPath = subParts.join('\\');
					if (fullPath.length === 2 && fullPath.endsWith(':')) fullPath += '\\';
				} else {
					fullPath = '/' + subParts.join('/');
				}

				return { name: curr, path: fullPath };
			})
			.filter(Boolean) as { name: string; path: string }[];

		return [{ name: rootName, path: rootPath }, ...segments];
	}

	const desktopSegments = $derived(createSegments(desktop.path, 'desktop'));
	const adbSegments = $derived(createSegments(adb.path, 'adb'));

	const visibleDesktopFiles = $derived(
		desktop.showHidden ? desktop.files : desktop.files.filter((f) => !f.name.startsWith('.'))
	);

	async function fetchPartitions() {
		partitions = await invoke<Partition[]>('list_partitions');
	}

	async function togglePartitionMenu() {
		if (!showPartitionMenu) {
			await fetchPartitions();
		}
		showPartitionMenu = !showPartitionMenu;
	}

	async function selectPartition(path: string) {
		showPartitionMenu = false;
		await navigateDesktop(path);
	}

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
		fetchPartitions();
		const unlisten = listen<DeviceObj[]>('adb_update', async (event) => {
			const newDevices = event.payload;
			const hasChanged = JSON.stringify(newDevices) !== JSON.stringify(adb.devices);

			if (hasChanged) {
				adb.devices = newDevices;

				if (newDevices.length > 0) {
					if (!adb.serial || !newDevices.find((d) => d.serial === adb.serial)) {
						adb.serial = newDevices[0].serial;
						await navigateAdb(adb.path);
					}
				} else {
					adb.serial = null;
					adb.files = [];
				}
			}
		});

		navigateDesktop(desktop.path);

		return () => {
			unlisten.then((f) => f());
		};
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

						<div class="relative">
							<button
								onclick={togglePartitionMenu}
								class="hover:bg-accent flex items-center gap-1 rounded-md px-2 py-1 text-sm font-semibold transition-colors {showPartitionMenu ? 'bg-accent' : ''}"
							>
								Partitions
								<ChevronDown
									size={14}
									class="opacity-50 transition-transform {showPartitionMenu ? 'rotate-180' : ''}"
								/>
							</button>

							{#if showPartitionMenu}
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_static_element_interactions -->
								<div class="fixed inset-0 z-40" onclick={() => (showPartitionMenu = false)}></div>

								<div
									class="bg-popover absolute top-full left-0 z-50 mt-1 w-56 rounded-md border p-1 shadow-lg"
								>
									<div class="text-muted-foreground px-2 py-1.5 text-[10px] font-bold uppercase">
										Available Drives
									</div>

									<div class="max-h-[300px] overflow-y-auto">
										{#each partitions as part (part.mount_point)}
											<button
												class="hover:bg-accent flex w-full items-center gap-2 rounded-sm px-2 py-2 text-left text-sm transition-colors"
												onclick={() => selectPartition(part.mount_point)}
											>
												<Folder size={14} class="text-blue-400" />
												<div class="flex flex-col leading-tight">
													<span class="truncate">{part.name || 'Local Disk'}</span>
													<span class="text-muted-foreground text-[10px]">{part.mount_point}</span>
												</div>
											</button>
										{/each}

										{#if partitions.length === 0}
											<button
												class="w-full px-2 py-2 text-left text-sm italic opacity-50"
												onclick={() => selectPartition('/')}
											>
												Root (/)
											</button>
										{/if}
									</div>
								</div>
							{/if}
						</div>
					</div>
					<div class="flex items-center gap-2">
						<Button
							variant="ghost"
							size="icon"
							class="h-7 w-7 {desktop.showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
							onclick={() => (desktop.showHidden = !desktop.showHidden)}
						>
							{#if desktop.showHidden}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
						</Button>

						<Navigation
							currentPath={desktop.path}
							segments={desktopSegments}
							onNavigate={navigateDesktop}
							type="desktop"
						/>
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

						<Navigation
							currentPath={adb.path}
							segments={adbSegments}
							onNavigate={navigateAdb}
							type="adb"
							disabled={!adb.serial}
						/>
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
			{#if adb.serial}
				<div class="h-2 w-2 rounded-full bg-green-500"></div>
			{:else}
				<div class="h-2 w-2 rounded-full bg-red-500"></div>
			{/if}
			<span class="text-muted-foreground">ADB: {adb.serial ?? 'Not Connected'}</span>
		</div>
	</div>
	<!-- #endregion -->
</div>
