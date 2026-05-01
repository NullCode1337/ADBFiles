export interface FileItem {
    name: string;
    path: string;
    is_dir: boolean;
    has_permission?: boolean;
}

export interface DeviceObj {
    serial: string;
    state: string;
}

export interface Partition {
    name: string;
    mount_point: string;
}

export interface PathMetadata {
    segments: Array<{ name: string; path: string }>;
    parent: string | null;
}
