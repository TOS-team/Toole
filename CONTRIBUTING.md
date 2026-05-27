# Contribuer à Toolé

## Commencer

1. Lire la documentation :
   - [ARCHITECTURE.md](docs/ARCHITECTURE.md) — architecture technique
   - [PROTOCOL.md](docs/PROTOCOL.md) — protocole réseau
   - [SECURITY.md](docs/SECURITY.md) — sécurité et TLS
   - [ROADMAP.md](docs/ROADMAP.md) — roadmap

2. Configurer l'environnement :
   - Installer Rust : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - Installer Node.js
   - Installer les dépendances Tauri (voir README.md)

3. Builder le projet :

```bash
cargo build
```

4. Lancer les tests :

```bash
cargo test
```

---

## Structure du projet

```
toole/
├── apps/
│   ├── desktop-ui/         # Interface Tauri (React + TypeScript)
│   └── cli/                # CLI (optionnel)
├── crates/
│   ├── discovery/          # UDP broadcast, découverte réseau
│   ├── transport/          # TCP, connexions multiples
│   ├── protocol/           # Packets, messages, sérialisation (serde)
│   ├── security/           # TLS (rustls), certificats, hash
│   ├── transfer/           # Chunking, progression, écriture disque
│   ├── core/               # Logique métier, orchestration
│   ├── app-state/          # État global (appareils, transferts, sessions)
│   └── ui-bridge/          # Communication Tauri (invoke / events)
├── docs/
├── tests/
├── scripts/
└── Cargo.toml
```

---

## Conventions de branches

```
main
develop
feature/discovery
feature/tls
feature/transfer
```

---

## Conventions de commits

Utiliser le format suivant :

```
feat(discovery): add udp broadcast listener
```

---

## Pull Requests

Chaque fonctionnalité doit avoir :
- une branche dédiée
- une Pull Request
- une review

---

## Procédure

1. Fork le projet
2. Créer une branche : `git checkout -b feature/ma-fonctionnalite`
3. Coder en suivant les conventions
4. Vérifier que les tests passent : `cargo test`
5. Ouvrir une Pull Request

---

## Licence

En contribuant, vous acceptez que votre code soit distribué sous [LICENCE MIT](LICENSE).
