import { ArrowLeft, Folder, ImageIcon, FileCode, FileText, VideoIcon, File } from '@lucide/svelte';
import type { Component } from 'svelte';
import type { FileItem } from '$lib/types';

export function getFileIcon(file: FileItem): Component {
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
