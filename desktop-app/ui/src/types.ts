export interface Peer {
  id: string;
  addr: string;
}

export interface FileEntry {
  path: string;
  name: string;
  size?: number;
  isDir?: boolean;
}
