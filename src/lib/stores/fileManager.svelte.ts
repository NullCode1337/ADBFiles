import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { join, tempDir } from '@tauri-apps/api/path';
import { platform } from '@tauri-apps/plugin-os';
import type { FileItem, DeviceObj, Partition, PathMetadata } from '$lib/types';

function createFileManagerStore() {
	const desktop = $state({
		path: localStorage.getItem('lastDesktopPath') ?? '',
		files: [] as FileItem[],
		meta: { segments: [], parent: null } as PathMetadata
	});

	const adb = $state({
		path: '/storage/emulated/0',
		files: [] as FileItem[],
		serial: null as string | null,
		devices: [] as DeviceObj[],
		meta: { segments: [], parent: null } as PathMetadata,
		name: 'Android Device'
	});

	let showHidden = $state(false);
	let partitions = $state([] as Partition[]);
	const partitionHistory = $state<Record<string, string>>(
		JSON.parse(localStorage.getItem('partitionHistory') ?? '{}')
	);

	// Derived states
	const activeDevice = $derived(adb.devices.find((d) => d.state === 'device'));

	const visibleDesktopFiles = $derived.by(() => {
		let list = showHidden ? desktop.files : desktop.files.filter((f) => !f.name.startsWith('.'));

		if (desktop.meta.parent) {
			list = [
				{ name: '..', path: desktop.meta.parent, is_dir: true, has_permission: true },
				...list
			];
		}
		return list;
	});

	const visibleAdbFiles = $derived.by(() => {
		let list = showHidden ? adb.files : adb.files.filter((f) => !f.name.startsWith('.'));

		if (adb.meta.parent) {
			list = [{ name: '..', path: adb.meta.parent, is_dir: true, has_permission: true }, ...list];
		}
		return list;
	});

	const partLabel = $derived.by(() => {
		const match = partitions
			.filter((p) => desktop.path.startsWith(p.mount_point))
			.sort((a, b) => b.mount_point.length - a.mount_point.length)[0];
		return match ? match.name || match.mount_point : 'Partitions';
	});

	// Navigate desktop, adb
	async function navigateDesktop(path: string) {
		try {
			const [files, meta] = await Promise.all([
				invoke<FileItem[]>('list_directory', { path }),
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
				invoke<FileItem[]>('list_adb_directory', { serial: adb.serial, path }),
				invoke<PathMetadata>('get_path_metadata', { path, isAdb: true })
			]);

			adb.files = files;
			adb.path = path;
			adb.meta = meta;
		} catch (err) {
			console.error('ADB Navigation failed:', err);
		}
	}

	// File open, delete, transfer
	async function openFile(file: FileItem, type: 'desktop' | 'adb') {
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
			alert(`Could not open file: ${err}`);
		}
	}

	async function deleteFile(file: FileItem, type: 'desktop' | 'adb') {
		if (!confirm(`Are you sure you want to delete ${file.name}?`)) return;
		try {
			if (type === 'desktop') {
				await invoke('delete_desktop_file', { path: file.path });
				await navigateDesktop(desktop.path);
			} else {
				if (!adb.serial) return;
				await invoke('delete_adb_file', { serial: adb.serial, path: file.path });
				await navigateAdb(adb.path);
			}
			await invoke('notify', { body: `Deleted ${file.name} successfully` });
		} catch (err) {
			alert(`Delete failed: ${err}`);
		}
	}

	async function transferFile(file: FileItem, type: 'desktop' | 'adb') {
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
			alert(`Transfer failed: ${err}`);
		}
	}

	// ADB helper utils
	async function refreshDevices() {
		try {
			const devices = await invoke<DeviceObj[]>('list_adb_devices');
			adb.devices = devices;
			if (devices.length > 0) {
				adb.serial = devices[0].serial;
				adb.name = devices[0].name;
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

	// Partitions
	async function fetchPartitions() {
		partitions = await invoke<Partition[]>('list_partitions');
	}

	// Drag and Drop
	async function dropOpen(path: string, isDir: boolean) {
		if (isDir) await navigateDesktop(path);
	}

	async function dropPush(path: string, name: string, isDir: boolean) {
		if (!adb.serial) return;
		try {
			await invoke('adb_push', { serial: adb.serial, src: path, dest: adb.path, isDir });
			await navigateAdb(adb.path);
			await invoke('notify', { body: `Pushed ${name} to device.` });
		} catch (err) {
			alert(`Transfer failed: ${err}`);
		}
	}

	// On mount ( init )
	async function init() {
		if (!desktop.path) {
			const p = await platform();
			desktop.path = p === 'windows' ? 'C:\\' : '/';
		}
		await fetchPartitions();
		await navigateDesktop(desktop.path);
		await refreshDevices();

		const unlisten = await listen<DeviceObj[]>('adb_update', (event) => {
			const devices = event.payload;
			adb.devices = devices;

			const currentDevice = devices.find((d) => d.serial === adb.serial);

			if (!currentDevice || currentDevice.state !== 'device') {
				// If the device we were browsing disconnected,
				// try to auto-switch to the next available 'device'
				const nextDevice = devices.find((d) => d.state === 'device');
				if (nextDevice) {
					adb.serial = nextDevice.serial;
					adb.name = nextDevice.name;
					navigateAdb(adb.path);
				} else {
					adb.serial = null;
					adb.name = 'No Device';
					adb.files = [];
				}
			} else {
				adb.name = currentDevice.name;
			}
		});

		return unlisten;
	}

	return {
		get desktop() {
			return desktop;
		},
		get adb() {
			return adb;
		},
		get showHidden() {
			return showHidden;
		},
		set showHidden(v: boolean) {
			showHidden = v;
		},
		get partitions() {
			return partitions;
		},
		get partitionHistory() {
			return partitionHistory;
		},
		get activeDevice() {
			return activeDevice;
		},
		get visibleDesktopFiles() {
			return visibleDesktopFiles;
		},
		get visibleAdbFiles() {
			return visibleAdbFiles;
		},
		get partLabel() {
			return partLabel;
		},
		navigateDesktop,
		navigateAdb,
		openFile,
		deleteFile,
		transferFile,
		refreshDevices,
		startScrcpy,
		fetchPartitions,
		dropOpen,
		dropPush,
		init
	};
}

export const fm = createFileManagerStore();
