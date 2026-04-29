export interface PathSegment {
    name: string;
    path: string;
}

export function parsePath(path: string, type: 'desktop' | 'adb' = 'desktop'): {
    segments: PathSegment[];
    parent: string | null;
    isWindows: boolean;
} {
    const isWindows = type === 'desktop' && (/^[a-zA-Z]:/.test(path) || path.includes('\\'));
    
    const normalized = path.replace(/\\/g, '/');
    const parts = normalized.split('/').filter(Boolean);
    
    let rootName = 'root';
    let rootPath = '/';
    
    if (isWindows) {
        const drive = path.match(/^([a-zA-Z]:)/)?.[1] || parts[0];
        rootName = drive;
        rootPath = drive + '\\';
    }

    const segments: PathSegment[] = [{ name: rootName, path: rootPath }];
    
    parts.forEach((part, i) => {
        if (isWindows && i === 0) return;

        const subParts = parts.slice(0, i + 1);
        let fullPath: string;

        if (isWindows) {
            fullPath = subParts.join('\\');
            if (fullPath.length === 2 && fullPath.endsWith(':')) fullPath += '\\';
        } else {
            fullPath = '/' + subParts.join('/');
        }

        segments.push({ name: part, path: fullPath });
    });

    let parent: string | null = null;
    if (isWindows) {
        if (parts.length > 1) {
            const parentParts = parts.slice(0, -1);
            parent = parentParts.join('\\');
            if (parent.length === 2 && parent.endsWith(':')) parent += '\\';
        }
    } else {
        if (parts.length > 0) {
            parent = '/' + parts.slice(0, -1).join('/');
        }
    }

    return { segments, parent, isWindows };
}