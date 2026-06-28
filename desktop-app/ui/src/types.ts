export interface Peer {
  hostname: string;
  addr: string;
}

export interface FileEntry {
  path: string;
  name: string;
  size?: number;
}
