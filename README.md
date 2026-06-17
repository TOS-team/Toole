# Toolé

![Toolé](assets/banner.png)

**Transfert de fichiers P2P en WiFi ad hoc, sans Internet.**

Toolé est un logiciel open source de transfert de fichiers pair-à-pair (P2P) fonctionnant par WiFi ad hoc, sans réseau existant, sans clé USB, sans compte cloud.

Deux clics : un sur **Envoyer**, un sur **Recevoir**, et le transfert démarre.

---

## Fonctionnalités

- Création automatique d'un hotspot WiFi ouvert `Toole-XXXX`
- Scan en boucle des réseaux Toolé disponibles
- Mode **Send** : devient hôte, diffuse sa présence
- Mode **Receive** : scanne, se connecte, reçoit
- Transfert sécurisé via **TLS 1.3** avec chiffrement bout-en-bout
- Vérification d'intégrité : **SHA-256** fichier final
- Accusé de réception (**Ack**) par chunk pour fiabilité
- Streaming mémoire optimisé : pas de chargement complet en RAM
- Interface minimaliste en HTML/CSS/JS vanilla

---

## Stack technologique

| Domaine | Technologie |
|---|---|
| Langage | Rust |
| Runtime async | Tokio |
| Interface desktop | Tauri v2 |
| Frontend | HTML / CSS / JS vanilla |
| Réseau ad hoc | nmcli |
| Chiffrement | rustls (TLS 1.3) |
| Hash SHA-256 | sha2 |
| Sérialisation | serde + serde_json |
| Logging | tracing |

---

## Prérequis

- [Rust](https://www.rust-lang.org) — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [Tauri CLI](https://tauri.app) — `cargo install tauri-cli`
- NetworkManager (`nmcli`) — préinstallé sur la plupart des distributions Linux

## Documentation

| Document | Description |
|---|---|
| [architecture.md](docs/architecture.md) | Architecture technique complète |
| [protocol.md](docs/protocol.md) | Protocole réseau détaillé |
| [security.md](docs/security.md) | Sécurité, TLS, SHA-256 |
| [roadmap.md](docs/roadmap.md) | Roadmap de développement |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Guide de contribution |

---

## Build

```bash
cargo build
```

## Dev

```bash
cargo tauri dev
```

---

## Licence

Licence MIT — voir le fichier [LICENSE](LICENSE) pour plus de détails.
