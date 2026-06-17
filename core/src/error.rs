// Types d'erreurs de Toolé.
//
// ToolError enum avec les variantes :
//   NoWifiInterface    — aucune interface WiFi trouvée
//   Nmcli(String)      — erreur nmcli
//   Io(std::io::Error) — erreur d'entrée/sortie
//   Tls(String)        — erreur TLS (rustls)
//   Sha256Mismatch     — SHA-256 du fichier reçu ne correspond pas
//   Timeout            — délai dépassé
//   Cancelled          — annulé par l'utilisateur
//   Custom(String)     — message personnalisé
//
// Doit implémenter std::error::Error, fmt::Display, et From<std::io::Error>.
