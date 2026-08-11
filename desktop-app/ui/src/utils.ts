// je formate une taille en octets avec des unités décimales (×1000), comme
// le font les systèmes de fichiers : 1024 octets = 1.0 Ko, pas 1 KiB
export function formatSize(bytes: number): string {
  if (bytes < 1000) return `${bytes} o`;
  if (bytes < 1000 * 1000) return `${(bytes / 1000).toFixed(1)} Ko`;
  if (bytes < 1000 * 1000 * 1000)
    return `${(bytes / (1000 * 1000)).toFixed(1)} Mo`;
  return `${(bytes / (1000 * 1000 * 1000)).toFixed(1)} Go`;
}

// je liste les extensions que je sais reconnaître pour choisir une icône
const IMAGE_EXT = new Set(["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg", "avif"]);
const DOC_EXT = new Set(["pdf", "txt", "md", "doc", "docx", "odt", "html", "htm", "csv", "xls", "xlsx", "ppt", "pptx"]);
const ARCHIVE_EXT = new Set(["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "zst"]);
const VIDEO_EXT = new Set(["mp4", "mkv", "webm", "avi", "mov", "m4v", "wmv", "flv"]);
const AUDIO_EXT = new Set(["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus"]);
const CODE_EXT = new Set(["rs", "js", "ts", "jsx", "tsx", "py", "java", "c", "cpp", "h", "hpp", "go", "rb", "php", "sh", "json", "toml", "yaml", "yml", "xml", "sql", "css", "scss", "vue", "html"]);

// je renvoie l'extension d'un nom de fichier (sans le point, en minuscules),
// ou une chaîne vide si le nom n'en a pas
export function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  if (i <= 0) return "";
  return name.slice(i + 1).toLowerCase();
}

// je décris visuellement un fichier : icône à afficher et miniature ou non
export interface FileVisual {
  icon: string;
  thumb: boolean;
}

// icone ou miniature associee a un fichier selon son type
export function fileVisual(name: string): FileVisual {
  const ext = extOf(name);
  if (IMAGE_EXT.has(ext)) return { icon: "image", thumb: true };
  if (VIDEO_EXT.has(ext)) return { icon: "movie", thumb: false };
  if (AUDIO_EXT.has(ext)) return { icon: "music", thumb: false };
  if (ARCHIVE_EXT.has(ext)) return { icon: "folder-zip", thumb: false };
  if (CODE_EXT.has(ext)) return { icon: "code", thumb: false };
  if (DOC_EXT.has(ext)) return { icon: "document", thumb: false };
  return { icon: "file", thumb: false };
}
