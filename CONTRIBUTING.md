# Contribuer à Toolé

## Commencer

1. Lire la documentation dans cet ordre :
   - [prd.md](docs/prd.md) — les exigences produit
   - [srs.md](docs/srs.md) — les spécifications logicielles
   - [architecture.md](docs/architecture.md) — architecture technique
   - [protocol.md](docs/protocol.md) — protocole réseau
   - [crypto.md](docs/crypto.md) — chiffrement et TLS
   - [roadmap.md](docs/roadmap.md) — roadmap

2. Configurer l'environnement :
   - Installer Rust : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - Installer Tauri CLI : `cargo install tauri-cli`

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
├── core/                       # Bibliothèque Rust pure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Trait UI + types
│       ├── error.rs            # ToolError
│       ├── utils.rs            # Fonctions utilitaires
│       ├── discovery.rs        # UDP broadcast
│       ├── transfer.rs         # TCP + TLS + chunks + SHA-256
│       └── network.rs          # nmcli hotspot / scan / connexion
│
├── app/                        # Application Tauri
│   ├── Cargo.toml
│   ├── src/
│   │   ├── index.html
│   │   ├── main.js
│   │   └── style.css
│   └── src-tauri/
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       ├── capabilities/default.json
│       ├── build.rs
│       ├── icons/
│       └── src/
│           ├── main.rs
│           └── commands.rs
│
├── docs/
├── assets/
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

---

## Conventions de branches

```
main
develop
feature/network
feature/discovery
feature/transfer
feature/security
```

---

## Conventions de commits

Utiliser le format suivant :

```
feat(network): add hotspot creation via nmcli
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
