<script lang="ts">
  	import { ArrowRight, ArrowLeft, Monitor, Smartphone, RefreshCw, Folder, File, FileText, ImageIcon, FileCode, Lock, SunIcon, MoonIcon, VideoIcon, Eye, EyeOff, ChevronDown, Trash2 } from "@lucide/svelte";

	import * as Resizable from '$lib/components/ui/resizable';
	import { Button } from '$lib/components/ui/button';

	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { join, tempDir } from '@tauri-apps/api/path';
	import { platform } from '@tauri-apps/plugin-os';

	import { ModeWatcher, toggleMode } from 'mode-watcher';
	import { onMount } from 'svelte';

	import Navigation from '$lib/components/Navigation.svelte';
	import VirtualList from '$lib/components/VirtualList.svelte';
	import FileDropper from '$lib/components/FileDropper.svelte';

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

	interface PathMetadata {
		segments: Array<{ name: string; path: string }>;
		parent: string | null;
	}

	let desktop = $state({
		path: localStorage.getItem('lastDesktopPath') ?? '',
		files: [] as File[],
		meta: { segments: [], parent: null } as PathMetadata
	});

	let adb = $state({
		path: '/storage/emulated/0',
		files: [] as File[],
		serial: null as string | null,
		devices: [] as DeviceObj[],
		meta: { segments: [], parent: null } as PathMetadata
	});
	const activeDevice = $derived(adb.devices.find((d) => d.state === 'device'));

	let showHidden = $state(false);
	let partitions = $state([] as Partition[]);
	let partitionHistory = $state<Record<string, string>>(
		JSON.parse(localStorage.getItem('partitionHistory') ?? '{}')
	);
	let showPartitionMenu = $state(false);

	const visibleDesktopFiles = $derived(
		(() => {
			let list = showHidden ? desktop.files : desktop.files.filter((f) => !f.name.startsWith('.'));

			if (desktop.meta.parent) {
				list = [
					{
						name: '..',
						path: desktop.meta.parent,
						is_dir: true,
						has_permission: true
					} as File,
					...list
				];
			}
			return list;
		})()
	);

	const visibleAdbFiles = $derived(
		(() => {
			let list = showHidden ? adb.files : adb.files.filter((f) => !f.name.startsWith('.'));

			if (adb.meta.parent) {
				list = [
					{
						name: '..',
						path: adb.meta.parent,
						is_dir: true,
						has_permission: true
					} as File,
					...list
				];
			}
			return list;
		})()
	);

	const partLabel = $derived(() => {
		const match = partitions
			.filter((p) => desktop.path.startsWith(p.mount_point))
			.sort((a, b) => b.mount_point.length - a.mount_point.length)[0];

		return match ? match.mount_point : 'Partitions';
	});

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

	function scrollPartitions(event: WheelEvent) {
		if (partitions.length === 0) return;

		const current =
			partitions
				.map((p, index) => ({ p, index }))
				.filter(({ p }) => desktop.path.startsWith(p.mount_point))
				.sort((a, b) => b.p.mount_point.length - a.p.mount_point.length)[0]?.index ?? 0;

		const direction = event.deltaY > 0 ? 1 : -1;
		const next = (current + direction + partitions.length) % partitions.length;
		const path = partitionHistory[partitions[next].mount_point] ?? partitions[next].mount_point;

		navigateDesktop(path);
	}

	function getFileIcon(file: File) {
		if (file.name === '..') return ArrowLeft;
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
			const [files, meta] = await Promise.all([
				invoke<File[]>('list_directory', { path }),
				invoke<PathMetadata>('get_path_metadata', { path, isAdb: false })
			]);

			desktop.files = files;
			desktop.path = path;
			desktop.meta = meta;
			localStorage.setItem('lastDesktopPath', path);

			const ownerPartition = partitions
				.filter((p) => path.startsWith(p.mount_point))
				.sort((a, b) => b.mount_point.length - a.mount_point.length)[0];

			if (ownerPartition) {
				partitionHistory[ownerPartition.mount_point] = path;
				localStorage.setItem('partitionHistory', JSON.stringify(partitionHistory));
			}
		} catch (err: unknown) {
			const error = err instanceof Error ? err.message : String(err);
			if (error.includes('os error 2')) {
				const root = (await platform()) === 'windows' ? 'C:\\' : '/';
				if (path !== root) await navigateDesktop(root);
			} else {
				alert(`Error: ${error}`);
			}
		}
	}

	async function navigateAdb(path: string) {
		if (!adb.serial) return;
		try {
			const [files, meta] = await Promise.all([
				invoke<File[]>('list_adb_directory', { serial: adb.serial, path }),
				invoke<PathMetadata>('get_path_metadata', { path, isAdb: true })
			]);

			adb.files = files;
			adb.path = path;
			adb.meta = meta;
		} catch (err) {
			console.error('ADB Navigation failed:', err);
		}
	}

	async function onClick(file: File, type: 'desktop' | 'adb') {
		if (file.is_dir) return;

		try {
			if (type === 'desktop') {
				await invoke('open_file', { path: file.path });
			} else {
				if (!adb.serial) return;

				const tempPath = await tempDir();
				await invoke('adb_pull', {
					serial: adb.serial,
					src: file.path,
					dest: tempPath,
					isDir: false
				});

				const localPath = await join(tempPath, file.name);

				await invoke('open_file', { path: localPath });
				await invoke('notify', { body: `Opened ${file.name}` });
			}
		} catch (err) {
			console.error('Failed to open file:', err);
			alert(`Could not open file: ${err}`);
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

	async function deleteFile(file: File, type: 'desktop' | 'adb') {
		const confirmed = confirm(`Are you sure you want to delete ${file.name}?`);
		if (!confirmed) return;

		try {
			if (type === 'desktop') {
				await invoke('delete_desktop_file', { path: file.path });
				await navigateDesktop(desktop.path);
			} else {
				if (!adb.serial) return;
				await invoke('delete_adb_file', {
					serial: adb.serial,
					path: file.path
				});
				await navigateAdb(adb.path);
			}

			await invoke('notify', { body: `Deleted ${file.name} successfully` });
		} catch (err) {
			alert(`Delete failed: ${err}`);
		}
	}

	async function transferFiles(file: File, type: 'desktop' | 'adb') {
		if (!adb.serial) return;

		try {
			if (type === 'desktop') {
				await invoke('adb_push', {
					serial: adb.serial,
					src: file.path,
					dest: adb.path,
					isDir: file.is_dir
				});
				await navigateAdb(adb.path);
				await invoke('notify', { body: `Pushed ${file.name} to device.` });
			} else {
				await invoke('adb_pull', {
					serial: adb.serial,
					src: file.path,
					dest: desktop.path,
					isDir: file.is_dir
				});
				await navigateDesktop(desktop.path);
				await invoke('notify', { body: `Pulled ${file.name} from device.` });
			}
		} catch (err) {
			console.log(err);
			alert(`Transfer failed: ${err}`);
		}
	}

	$effect(() => {
		const serial = activeDevice?.serial ?? null;

		if (serial !== adb.serial) {
			adb.serial = serial;
			if (serial) {
				navigateAdb(adb.path);
			} else {
				adb.files = [];
			}
		}
	});

	async function dropOpen(path: string, isDir: boolean) {
		if (isDir) {
			await navigateDesktop(path);
		}
	}

	async function dropPush(path: string, name: string, isDir: boolean) {
		if (!adb.serial) return;
		try {
			await invoke('adb_push', {
				serial: adb.serial,
				src: path,
				dest: adb.path,
				isDir: isDir
			});
			await navigateAdb(adb.path);
			await invoke('notify', { body: `Pushed ${name} to device.` });
		} catch (err) {
			alert(`Transfer failed: ${err}`);
		}
	}

	onMount(() => {
		const setup = async() => {
			if (!desktop.path) {
				const p = await platform();
				desktop.path = p === 'windows' ? 'C:\\' : '/';
			}
			fetchPartitions();
			navigateDesktop(desktop.path);
		}
	
		setup();
		
		// Refresh ADB once before polling as no update when device pre-connected
		refreshDevices();
		// ADB Polling (rust backend sends event)
		const unlisten = listen<DeviceObj | DeviceObj[]>('adb_update', (event) => {
			const payload = Array.isArray(event.payload) ? event.payload : [event.payload];

			payload.forEach((incoming) => {
				const index = adb.devices.findIndex((d) => d.serial === incoming.serial);

				if (index !== -1) {
					adb.devices[index] = {
						...adb.devices[index],
						state: incoming.state
					};
				} else {
					adb.devices.push(incoming);
				}
			});
		});

		return () => {
			unlisten.then((f) => f());
		};
	});
</script>

<ModeWatcher />

<FileDropper 
    adbSerial={adb.serial} 
    onOpen={dropOpen}
    onPush={dropPush}
/>

{#snippet file_list(
	files: File[],
	onNavigate: (path: string) => Promise<void>,
	type: 'desktop' | 'adb'
)}
	<VirtualList items={files} itemHeight={38}>
		{#snippet children(file: File)}
			{@const Icon = getFileIcon(file)}
			<div class="group flex h-full items-center gap-1 px-4">
				<button
					onclick={() => {
						if (file.is_dir && (file.has_permission ?? true)) {
							onNavigate(file.path);
						} else {
							onClick(file, type);
						}
					}}
					disabled={file.has_permission === false}
					class="flex min-w-0 flex-1 items-center rounded-md p-1.5 text-sm transition-colors
                    {(file.has_permission ?? true)
						? 'hover:bg-accent cursor-pointer'
						: 'cursor-not-allowed opacity-40'}"
				>
					<div class="flex min-w-0 flex-1 items-center gap-3">
						<Icon
							size={16}
							class="shrink-0 {file.is_dir
								? type === 'desktop'
									? 'text-blue-400'
									: 'text-green-400'
								: 'text-zinc-400'}"
						/>
						<span class="block truncate text-left">{file.name}</span>
					</div>
					{#if file.has_permission === false}
						<Lock size={12} class="text-muted-foreground ml-2 shrink-0" />
					{/if}
				</button>

				{#if file.has_permission !== false && file.name !== '..'}
					<div
						class="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100"
					>
						{#if adb.serial}
							<Button
								variant="ghost"
								size="icon"
								class="h-7 w-7 hover:bg-blue-500/10 cursor-pointer {type === 'adb' ? 'text-green-500' : 'text-blue-500'}"
								onclick={(e) => {
									e.stopPropagation();
									transferFiles(file, type);
								}}
								title={type === 'desktop' ? 'Push to Device' : 'Pull from Device'}
							>
								{#if type === 'desktop'}
									<ArrowRight size={14} />
								{:else}
									<ArrowLeft size={14} />
								{/if}
							</Button>
						{/if}

						<Button
							variant="ghost"
							size="icon"
							class="text-destructive h-7 w-7 cursor-pointer hover:text-white"
							onclick={(e) => {
								e.stopPropagation();
								deleteFile(file, type);
							}}
						>
							<Trash2 size={14} />
						</Button>
					</div>
				{/if}
			</div>
		{/snippet}
	</VirtualList>
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
								onwheel={scrollPartitions}
								class="hover:bg-accent flex items-center gap-1 rounded-md px-2 py-1 text-sm font-semibold transition-colors {showPartitionMenu ? 'bg-accent' : ''}"
							>
								{partLabel()}
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
						<Navigation
							segments={desktop.meta.segments}
							parentPath={desktop.meta.parent}
							onNavigate={navigateDesktop}
							type="desktop"
						/>
					</div>
				</div>

				<div class="h-full w-full flex-1 overflow-hidden">
					{@render file_list(visibleDesktopFiles, navigateDesktop, 'desktop')}
				</div>
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
							segments={adb.meta.segments}
							parentPath={adb.meta.parent}
							onNavigate={navigateAdb}
							type="adb"
							disabled={!adb.serial}
						/>
					</div>
				</div>

				{#if adb.serial}
					<div class="h-full w-full flex-1 overflow-hidden">
						{@render file_list(visibleAdbFiles, navigateAdb, 'adb')}
					</div>
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
		<div class="flex items-center gap-4">
			<span>Files: {visibleDesktopFiles.length}</span>
			<Button
				variant="outline"
				size="icon"
				class="h-7 w-7 {showHidden ? 'text-blue-500' : 'text-muted-foreground'}"
				onclick={() => (showHidden = !showHidden)}
			>
				{#if showHidden}<Eye size={14} />{:else}<EyeOff size={14} />{/if}
			</Button>
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
