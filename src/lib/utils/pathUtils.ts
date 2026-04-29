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
    
    let normalized = path.replace(/\\/g, '/');
    if (normalized.length > 1 && normalized.endsWith('/')) {
        normalized = normalized.slice(0, -1);
    }

    const segments: PathSegment[] = [];

    if (isWindows) {
        const driveMatch = normalized.match(/^([a-zA-Z]:)/);
        const rootPath = driveMatch ? driveMatch[1] + '\\' : 'C:\\';
        segments.push({ name: rootPath.replace('\\', ''), path: rootPath });

        const parts = normalized.split('/').filter(Boolean);
        if (parts[0] && parts[0].toLowerCase() === rootPath.toLowerCase().replace('\\', '').replace(':', '')) {
            parts.shift();
        }

        let current = rootPath;
        parts.forEach(part => {
            current = current.endsWith('\\') ? current + part : current + '\\' + part;
            segments.push({ name: part, path: current });
        });
    } else {
        segments.push({ name: 'root', path: '/' });
        
        const parts = normalized.split('/').filter(Boolean);
        let current = '';
        parts.forEach(part => {
            current += '/' + part;
            segments.push({ name: part, path: current });
        });
    }

    let parent: string | null = null;

    if (segments.length > 1) {
        parent = segments[segments.length - 2].path;
    }

    return { 
        segments, 
        parent, 
        isWindows 
    };
}