import {
	ArrowLeft,
	Folder,
	ImageIcon,
	FileCode,
	FileText,
	VideoIcon,
	File,
	ArchiveIcon,
	MusicIcon,
	FileTerminal
} from '@lucide/svelte';
import type { Component } from 'svelte';
import type { FileItem } from '$lib/types';

export function getFileIcon(file: FileItem): Component {
	if (file.name === '..') return ArrowLeft;
	if (file.is_dir) return Folder;

	const ext = file.name.split('.').pop()?.toLowerCase() ?? '';
	const maps = {
		image: [
			'png',
			'jpg',
			'jpeg',
			'gif',
			'svg',
			'webp',
			'avif',
			'heic',
			'heif',
			'tiff',
			'bmp',
			'ico',
			'raw',
			'cr2',
			'nef',
			'psd',
			'ai',
			'xd',
			'fig',
			'sketch'
		],
		video: [
			'mp4',
			'm4v',
			'mov',
			'wmv',
			'avi',
			'mpeg',
			'mpg',
			'webm',
			'mkv',
			'flv',
			'vob',
			'ogv',
			'3gp'
		],
		code: ['ts', 'js', 'py', 'rs', 'c', 'cpp', 'json', 'bat', 'ps1', 'html', 'svelte'],
		text: [
			'txt',
			'md',
			'pdf',
			'doc',
			'docx',
			'ppt',
			'xlsx',
			'conf',
			'toml',
			'db',
			'sqlite',
			'sql',
			'mdb'
		],
		music: ['mp3', 'aac', 'ogg', 'wma', 'm4a', 'aiff', 'flac', 'alac', 'opus', 'wav'],
		archive: ['zip', 'rar', '7z', 'tar', 'gz', 'iso'],
		executable: ['exe', 'msi', 'bin', 'dmg', 'app', 'sh', 'bat']
	};
	if (maps.image.includes(ext)) return ImageIcon;
	if (maps.code.includes(ext)) return FileCode;
	if (maps.text.includes(ext)) return FileText;
	if (maps.music.includes(ext)) return MusicIcon;
	if (maps.video.includes(ext)) return VideoIcon;
	if (maps.archive.includes(ext)) return ArchiveIcon;
	if (maps.executable.includes(ext)) return FileTerminal;
	return File;
}
