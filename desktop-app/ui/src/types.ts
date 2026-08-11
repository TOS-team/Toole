// un appareil Toolé détecté sur le réseau local
export interface Peer {
  id: string; // identifiant stable de l'appareil (hostname + suffixe)
  addr: string; // adresse IP où joindre son port d'écoute
}

// un fichier choisi pour envoi (source locale)
export interface FileEntry {
  path: string; // chemin absolu sur la machine
  name: string; // nom affiché (basename)
  size?: number; // taille en octets, renseignée via get_file_infos
  isDir?: boolean; // vrai si c'est un dossier
}
